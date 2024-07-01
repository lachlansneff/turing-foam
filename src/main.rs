use std::collections::HashMap;

use rand::{
    distributions::{Distribution as _, Uniform},
    Rng,
};

const PROGRAM_LEN: usize = 64;

#[allow(dead_code)]
#[derive(Debug)]
struct TuringFoamStats {
    num_unique_tokens: usize,
    num_timed_out: usize,
    num_unmatched_branch: usize,
}

struct TuringFoam {
    programs: Vec<[u8; PROGRAM_LEN]>,
}

impl TuringFoam {
    pub fn new(num_programs: usize) -> Self {
        let mut rng = rand::thread_rng();

        let programs = (0..num_programs)
            .map(|_| {
                let mut program = [0; PROGRAM_LEN];
                rng.fill(&mut program[..]);
                program
            })
            .collect();

        Self { programs }
    }

    pub fn react(&mut self, num_reactions: usize) -> TuringFoamStats {
        let mut rng = rand::thread_rng();
        let indexes = Uniform::from(0..self.programs.len());
        let ips = Uniform::from(0..PROGRAM_LEN);

        let mut reaction = [0; PROGRAM_LEN * 2];
        let mut num_timed_out = 0;
        let mut num_unmatched_branch = 0;

        for _ in 0..num_reactions {
            let program0 = indexes.sample(&mut rng);
            let program1 = indexes.sample(&mut rng);

            reaction[..PROGRAM_LEN].copy_from_slice(&self.programs[program0]);
            reaction[PROGRAM_LEN..].copy_from_slice(&self.programs[program1]);

            match execute(&mut reaction, ips.sample(&mut rng), 10_000) {
                ProgramStatus::TimedOut => {
                    num_timed_out += 1;
                }
                ProgramStatus::UnmatchedBranch => {
                    num_unmatched_branch += 1;
                }
            }

            self.programs[program0].copy_from_slice(&reaction[..PROGRAM_LEN]);
            self.programs[program1].copy_from_slice(&reaction[PROGRAM_LEN..]);
        }

        let mut token_counts: HashMap<(u8, u8), usize> = HashMap::new();

        for program in &self.programs {
            for (i, c) in program.iter().enumerate() {
                *token_counts.entry((i as u8, *c)).or_default() += 1;
            }
        }

        TuringFoamStats {
            num_unique_tokens: token_counts.len(),
            num_timed_out,
            num_unmatched_branch,
        }
    }
}

enum ProgramStatus {
    TimedOut,
    UnmatchedBranch,
}

fn execute<const N: usize>(
    tape: &mut [u8; N],
    mut ip: usize,
    max_instructions: usize,
) -> ProgramStatus {
    let mut head0 = ip;
    let mut head1 = (ip + 16) % N;

    for _ in 0..max_instructions {
        match tape[ip] {
            b'<' => {
                head0 = (head0 - 1) % N;
            }
            b'>' => {
                head0 = (head0 - 1) % N;
            }
            b'{' => {
                head1 = (head1 - 1) % N;
            }
            b'}' => {
                head1 = (head1 + 1) % N;
            }
            b'-' => {
                tape[head0 % N] -= 1;
            }
            b'+' => {
                tape[head0 % N] += 1;
            }
            b'.' => {
                tape[head1 % N] = tape[head0 % N];
            }
            b',' => {
                tape[head0 % N] = tape[head1 % N];
            }
            b'[' => {
                if tape[head0] == 0 {
                    let mut depth: i32 = 0;
                    let Some(offset) = tape[ip..].iter().position(|&c| {
                        if c == b'[' {
                            depth += 1;
                        } else if c == b']' {
                            depth -= 1;
                        }

                        depth == 0
                    }) else {
                        return ProgramStatus::UnmatchedBranch;
                    };

                    ip += offset;
                }
            }
            b']' => {
                if tape[head0] != 0 {
                    let mut depth: i32 = 0;
                    let Some(offset) = tape[..=ip].iter().rev().position(|&c| {
                        if c == b']' {
                            depth += 1;
                        } else if c == b'[' {
                            depth -= 1;
                        }

                        depth == 0
                    }) else {
                        return ProgramStatus::UnmatchedBranch;
                    };

                    ip -= offset;
                }
            }
            _ => {}
        }

        ip = (ip + 1) % tape.len();
    }

    ProgramStatus::TimedOut
}

fn main() {
    let mut foam = TuringFoam::new(1 << 17);

    for i in 0..1000 {
        let stats = foam.react(1 << 17);

        println!("epoch {i}: {stats:?}");
    }
}
