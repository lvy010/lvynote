//! nano-scheduler — Rust port of the nano-vllm request scheduler.
//!
//! Mirrors the Python implementation documented in:
//!   note/机器学习/nano-vllm/3.md
//!
//! Core design (straight from the notes):
//!   • Two queues:  waiting → running
//!   • schedule()  — batches requests for the model runner
//!                   prefill phase first, then decode phase
//!   • postprocess() — appends generated tokens, marks finished requests
//!   • preempt()   — evicts a running request back to waiting when OOM
//!
//! The analogy from the notes: the scheduler is an air-traffic controller.
//! Sequences are flights; the GPU is the runway.

use std::collections::VecDeque;

// ── Status ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Waiting,
    Running,
    Finished,
}

// ── Request ──────────────────────────────────────────────────────────────────

/// Simplified version of nano-vllm's `Sequence`.
#[derive(Debug, Clone)]
pub struct Request {
    pub id: u64,
    pub prompt_tokens: Vec<u32>,
    pub generated_tokens: Vec<u32>,
    pub max_tokens: usize,
    pub status: Status,
    /// KV-cache blocks currently allocated for this request.
    pub num_blocks: usize,
}

impl Request {
    pub fn new(id: u64, prompt: Vec<u32>, max_tokens: usize) -> Self {
        let blocks = prompt.len().div_ceil(BLOCK_SIZE);
        Request {
            id,
            prompt_tokens: prompt,
            generated_tokens: Vec::new(),
            max_tokens,
            status: Status::Waiting,
            num_blocks: blocks,
        }
    }

    pub fn total_tokens(&self) -> usize {
        self.prompt_tokens.len() + self.generated_tokens.len()
    }

    pub fn is_finished(&self) -> bool {
        self.status == Status::Finished
    }
}

// Tokens per KV-cache block (mirrors PagedAttention block size).
const BLOCK_SIZE: usize = 16;

// ── Scheduler ────────────────────────────────────────────────────────────────

pub struct Scheduler {
    pub waiting: VecDeque<Request>,
    pub running: VecDeque<Request>,
    max_batch_tokens: usize, // max tokens in one forward pass
    max_seqs: usize,         // max concurrent sequences
    total_blocks: usize,     // total KV-cache blocks available
    used_blocks: usize,
    eos: u32,
}

impl Scheduler {
    pub fn new(max_batch_tokens: usize, max_seqs: usize, total_blocks: usize, eos: u32) -> Self {
        Scheduler {
            waiting: VecDeque::new(),
            running: VecDeque::new(),
            max_batch_tokens,
            max_seqs,
            total_blocks,
            used_blocks: 0,
            eos,
        }
    }

    /// Add a new request to the waiting queue (mirrors `Scheduler.add`).
    pub fn add(&mut self, req: Request) {
        self.waiting.push_back(req);
    }

    /// All work done?
    pub fn is_finished(&self) -> bool {
        self.waiting.is_empty() && self.running.is_empty()
    }

    fn free_blocks(&self) -> usize {
        self.total_blocks.saturating_sub(self.used_blocks)
    }

    fn allocate(&mut self, req: &mut Request) {
        self.used_blocks += req.num_blocks;
        req.status = Status::Running;
    }

    fn deallocate(&mut self, req: &Request) {
        self.used_blocks = self.used_blocks.saturating_sub(req.num_blocks);
    }

    /// Move a running request back to the *front* of the waiting queue.
    /// Mirrors `Scheduler.preempt` — freed memory lets other requests proceed.
    fn preempt(&mut self, mut req: Request) {
        self.deallocate(&req);
        req.status = Status::Waiting;
        self.waiting.push_front(req); // high priority: retry next round
    }

    /// Decide which requests to run next.
    ///
    /// Returns `(request_ids, is_prefill)`:
    ///   - `is_prefill = true`  → process full prompts (new / preempted reqs)
    ///   - `is_prefill = false` → generate one more token per running request
    ///
    /// Mirrors `Scheduler.schedule`.
    pub fn schedule(&mut self) -> (Vec<u64>, bool) {
        let mut batch: Vec<u64> = Vec::new();
        let mut batched_tokens: usize = 0;

        // ── Prefill phase ─────────────────────────────────────────────────
        // Admit as many waiting requests as batch size + memory allow.
        while let Some(front) = self.waiting.front() {
            let fits_tokens =
                batched_tokens + front.prompt_tokens.len() <= self.max_batch_tokens;
            // running already contains previously admitted seqs from this loop.
            let fits_seqs = self.running.len() < self.max_seqs;
            let fits_mem = self.free_blocks() >= front.num_blocks;

            if !fits_tokens || !fits_seqs || !fits_mem {
                break;
            }

            let mut req = self.waiting.pop_front().unwrap();
            batched_tokens += req.prompt_tokens.len();
            self.allocate(&mut req);
            batch.push(req.id);
            self.running.push_back(req);
        }

        if !batch.is_empty() {
            return (batch, true);
        }

        // ── Decode phase ──────────────────────────────────────────────────
        // Advance every running request by one token; preempt if OOM.
        let snapshot: Vec<Request> = self.running.drain(..).collect();

        for req in snapshot {
            if self.free_blocks() == 0 {
                // No room for the next KV-cache block → evict this request.
                self.preempt(req);
                continue;
            }
            batch.push(req.id);
            self.running.push_back(req);
        }

        (batch, false)
    }

    /// Incorporate newly generated tokens and retire finished requests.
    /// Mirrors `Scheduler.postprocess`.
    pub fn postprocess(&mut self, req_ids: &[u64], token_ids: &[u32]) {
        for (&rid, &tok) in req_ids.iter().zip(token_ids.iter()) {
            if let Some(req) = self.running.iter_mut().find(|r| r.id == rid) {
                req.generated_tokens.push(tok);

                // Allocate a new KV block every BLOCK_SIZE tokens.
                if req.total_tokens() % BLOCK_SIZE == 0 {
                    req.num_blocks += 1;
                    self.used_blocks += 1;
                }

                let hit_max = req.generated_tokens.len() >= req.max_tokens;
                let hit_eos = tok == self.eos;
                if hit_max || hit_eos {
                    req.status = Status::Finished;
                }
            }
        }

        // Collect finished requests and free their blocks.
        let done: Vec<Request> = self
            .running
            .iter()
            .filter(|r| r.is_finished())
            .cloned()
            .collect();
        for r in &done {
            self.deallocate(r);
        }
        self.running.retain(|r| !r.is_finished());
    }
}

// ── Demo ─────────────────────────────────────────────────────────────────────

fn main() {
    println!("nano-scheduler demo  (mirrors nano-vllm/3.md)\n");

    const EOS: u32 = 2;
    let mut sched = Scheduler::new(512, 4, 64, EOS);

    // Submit three requests with different prompt lengths.
    sched.add(Request::new(1, vec![10, 11, 12], 6));
    sched.add(Request::new(2, vec![20, 21], 4));
    sched.add(Request::new(3, vec![30], 3));

    let mut step = 0usize;
    while !sched.is_finished() {
        let (batch, is_prefill) = sched.schedule();
        let phase = if is_prefill { "prefill" } else { "decode " };
        print!("step {:02} [{}]  batch={:?}", step, phase, batch);

        if !batch.is_empty() && !is_prefill {
            // Simulate the model: emit EOS for the last request on step 3.
            let fake_tokens: Vec<u32> = batch
                .iter()
                .map(|&id| if step >= 3 && id == 1 { EOS } else { id as u32 + 100 })
                .collect();
            print!("  tokens={:?}", fake_tokens);
            sched.postprocess(&batch, &fake_tokens);
        }

        println!("  (waiting={}, running={})", sched.waiting.len(), sched.running.len());
        step += 1;

        if step > 20 {
            println!("(demo step limit reached)");
            break;
        }
    }

    println!("\nAll requests finished.");
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sched() -> Scheduler {
        Scheduler::new(512, 4, 64, 2)
    }

    #[test]
    fn first_schedule_is_prefill() {
        let mut s = sched();
        s.add(Request::new(1, vec![10, 11], 5));
        let (batch, is_prefill) = s.schedule();
        assert!(is_prefill);
        assert_eq!(batch, vec![1]);
    }

    #[test]
    fn decode_follows_prefill() {
        let mut s = sched();
        s.add(Request::new(1, vec![10], 4));
        s.schedule(); // prefill
        let (batch, is_prefill) = s.schedule();
        assert!(!is_prefill);
        assert_eq!(batch, vec![1]);
    }

    #[test]
    fn eos_finishes_request() {
        let mut s = sched();
        s.add(Request::new(1, vec![10], 100));
        s.schedule(); // prefill
        let (batch, _) = s.schedule();
        s.postprocess(&batch, &[2]); // EOS = 2
        assert!(s.is_finished());
    }

    #[test]
    fn max_tokens_finishes_request() {
        let mut s = sched();
        s.add(Request::new(1, vec![10], 2));
        s.schedule(); // prefill
        let (b1, _) = s.schedule();
        s.postprocess(&b1, &[99]);
        let (b2, _) = s.schedule();
        s.postprocess(&b2, &[99]); // 2 tokens generated → done
        assert!(s.is_finished());
    }

    #[test]
    fn respects_max_seqs() {
        let mut s = Scheduler::new(1024, 2, 128, 2);
        s.add(Request::new(1, vec![1], 5));
        s.add(Request::new(2, vec![2], 5));
        s.add(Request::new(3, vec![3], 5)); // held back
        let (batch, _) = s.schedule();
        assert_eq!(batch.len(), 2);
        assert_eq!(s.waiting.len(), 1);
    }

    #[test]
    fn preempt_on_memory_pressure() {
        // Only 1 block total: after prefill, decode phase has 0 free blocks.
        let mut s = Scheduler::new(1024, 8, 1, 2);
        s.add(Request::new(1, vec![1], 5)); // uses 1 block
        s.schedule(); // prefill → used_blocks = 1
        let (decode_batch, _) = s.schedule(); // decode → 0 free → preempted
        assert!(decode_batch.is_empty());
        assert_eq!(s.waiting.front().map(|r| r.id), Some(1));
    }
}
