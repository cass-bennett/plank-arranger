extern crate getopts;

use bitvec::prelude::*;
use getopts::Options;
use ordered_float::NotNan;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (max_p_len,file_name) = parse_args(&args[1..]);
    let pieces = get_pieces(&file_name);
    if *pieces.last().unwrap() > max_p_len {
        panic!("One or more of the pieces is too long for the planks.");
    }
    let soln = find_solution(max_p_len, &pieces);
    println!("\nSolution for the pieces found in file \"{}\", with plank length of {}:",file_name,max_p_len);
    for i in 0..soln.len() {
        print!("Plank {}:  ",(i+1));
        let p = &soln[i];
        for j in 0..p.len()-1 {
            print!("{}, ",*p[j]);
        }
        print!("{}\n",p.last().unwrap());
    }
    println!("");
}

/// Reads plank size and name of input file from command-line arguments, if
/// they're present, or prompts the user to input them otherwise
fn parse_args(args:&[String]) -> (f64,String) {
    let mut opts = Options::new();
    opts.optopt("p","plank-size","length of the planks that the pieces will be cut from","LENGTH");
    opts.optopt("f","file","file to read piece lengths from","FILE");
    let matches = match opts.parse(args) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    let size_of_plank = match matches.opt_str("p") {
        Some(s) => s,
        None => prompt_for_input("Enter the length of the planks: "),
    }.parse::<f64>().unwrap();
    let input_file = match matches.opt_str("f") {
        Some(s) => s,
        None => prompt_for_input("Enter name of file to read pieces from: "),
    };
    (size_of_plank,input_file)
}

/// Prompts the user for input and returns it
fn prompt_for_input(s:&'static str) -> String {
    print!("{}",s);
    io::stdout().flush().unwrap();
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).unwrap();
    // remove trailing newline
    user_input.pop();
    user_input
}

/// Reads the pieces from the specified file
fn get_pieces(file_name:&String) -> Vec<f64> {
    // read contents of the file to a vector of floats
    let mut file = File::open(file_name).expect("Couldn't open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Couldn't read the file");
    let mut v: Vec<f64> = contents.split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect::<Vec<_>>();
    // sort the numbers from low to high
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v.shrink_to_fit();
    v
}

/// Tries to arrange the elements of `pieces` into the smallest number of
/// `Vec`s possible, such that each `Vec` has a sum that's less than
/// or equal to `max_p_len`.
/// This function assumes that `pieces` will be sorted from smallest
/// to biggest.
fn find_solution(max_p_len: f64, pieces:&Vec<f64>) -> Vec<Vec<&f64>> {
    let mut planks: Vec<Plank> = Vec::new();
    planks.push(Plank::new(initial_num_pieces(&max_p_len,&pieces),
                           &bitvec![0;pieces.len()],
                           pieces));
    let mut rem_pieces = pieces.len() - planks.last().unwrap().num_pieces;
    let mut num_planks = (pieces.len() - 1) / (pieces.len() - rem_pieces) + 1;
    while ! (planks.len() == num_planks
             && planks.last().unwrap().small_enough(&max_p_len))
    {
        let mut curr_plank = planks.pop().unwrap();

        // if current plank is good, make the next one
        if curr_plank.small_enough(&max_p_len) {
            planks.push(curr_plank);
            // if this is the last plank, use all remaining pieces;
            // otherwise use the same number as the previous plank
            let new_num_pieces = if planks.len() == num_planks - 1 {
                rem_pieces } else { planks.last().unwrap().num_pieces };
            planks.push(Plank::new(new_num_pieces,
                                   &planks.last().unwrap().state_bv,
                                   pieces));
            rem_pieces -= planks.last().unwrap().num_pieces;
        } // if you can decrease # of pieces w/o increasing total # of planks
        else if (num_planks - planks.len()) * (curr_plank.num_pieces - 1)
            >= rem_pieces + curr_plank.num_pieces
        {
            curr_plank.set_num_pieces(curr_plank.num_pieces - 1);
            planks.push(curr_plank);
            rem_pieces += 1;
        } // if we have to get rid of the plank that was on top of the stack
        else {
            if ! planks.is_empty() {
                rem_pieces += curr_plank.num_pieces;
                curr_plank = planks.pop().unwrap();
                while ! curr_plank.has_next() && ! planks.is_empty() {
                    rem_pieces += curr_plank.num_pieces;
                    curr_plank = planks.pop().unwrap();
                }
            }
            if curr_plank.has_next() && curr_plank.small_enough(&max_p_len) {
                curr_plank.next();
                planks.push(curr_plank);
            } // if curr_plank is the last plank on the stack
            else {
                rem_pieces += 1;
                curr_plank.set_num_pieces(curr_plank.num_pieces-1);
                num_planks = (pieces.len() - 1) / (pieces.len() - rem_pieces) + 1;
                planks.push(curr_plank);
            }
        }
    }
    planks_to_vecs(planks)
}

/// Tries to find a value n such that the first n elements of `pieces`
/// have a sum less than or equal to `max_len`, but the first n+1 elements
/// have a sume greater than `max_len`.
fn initial_num_pieces(max_len: &f64, pieces: &Vec<f64>) -> usize {
    let mut total = 0.0;
    let mut count = 0;
    for p in pieces {
        total += *p;
        if total - *max_len < 0.00000001 {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Converts a set of `Plank`s into just the lengths of pieces on that `Plank`
fn planks_to_vecs(planks: Vec<Plank>) -> Vec<Vec<&f64>> {
    let mut ret = Vec::new();
    let mut prev_bv:BitVec = bitvec![0;planks.get(0).unwrap().all_pieces.len()];
    for p in planks {
        let curr_bv = p.state_bv.clone() ^ prev_bv;
        prev_bv = p.state_bv;
        let v = p.all_pieces.iter()
            .zip(curr_bv.iter())
            .filter_map(|(a,b)| if b { Some(a) } else { None })
            .collect::<Vec<_>>();
        ret.push(v);
    }
    ret
}

pub struct Plank<'a> {
    all_pieces: &'a Vec<f64>,
    usable_pieces: Vec<usize>,
    num_pieces: usize,
    total_len: f64,
    state_bv: BitVec,
    bit_queue: PriorityQueue<BitVec,Reverse<NotNan<f64>>>,
}

impl<'a> Plank<'a> {
    pub fn new(num_pieces: usize,
               prev_bits: &BitVec,
               pieces: &'a Vec<f64>) -> Plank<'a> {
        let mut available_pieces = Vec::new();
        let mut i = 0;
        while i < pieces.len() {
            if prev_bits.get(i) == Some(false) {
                available_pieces.push(i);
            }
            i += 1;
        }
        available_pieces.shrink_to_fit();
        let mut ret = Plank {
            all_pieces: pieces,
            usable_pieces: available_pieces,
            num_pieces: 0,
            total_len: 0.0,
            state_bv: prev_bits.clone(),
            bit_queue: PriorityQueue::new(),
        };
        ret.set_num_pieces(num_pieces);
        return ret;
    }

    /// Finds whether the sum of pieces on this plank is less than or
    /// equal to `max_len`
    fn small_enough(&self, max_len:&f64) -> bool {
        self.total_len - *max_len < 0.000000000001
    }

    /// Fills the `Plank` with the specified number of pieces, skipping
    /// over any pieces that are shown as being in use by `self.state_bv`
    fn set_num_pieces(&mut self, num_pieces: usize) {
        self.num_pieces = num_pieces;
        self.total_len = 0.0;
        self.bit_queue = PriorityQueue::new();

        // variables to find the leftmost and rightmost pieces whose lengths
        // equal that of the biggest piece we're adding
        let mut first_same = self.num_pieces - 1;
        let mut last_same = self.num_pieces - 1;
        while first_same > 0
            && self.nth_pieces_equal(&first_same, &(first_same-1)) {
            first_same -= 1;
        }
        while self.nth_pieces_equal(&last_same, &(last_same+1)) {
            last_same += 1;
        }
        let mid_same = last_same - (self.num_pieces - 1 - first_same);

        for i in 0..first_same {
            let index = self.usable_pieces.get(i).unwrap();
            self.total_len += *self.all_pieces.get(*index).unwrap();
            self.state_bv.set(*index,true);
        }
        for i in first_same..mid_same {
            let index = self.usable_pieces.get(i).unwrap();
            self.state_bv.set(*index,false);
        }
        for i in mid_same..last_same + 1 {
            let index = self.usable_pieces.get(i).unwrap();
            self.total_len += *self.all_pieces.get(*index).unwrap();
            self.state_bv.set(*index,true);
        }
        for i in last_same + 1..self.usable_pieces.len() {
            let index = self.usable_pieces.get(i).unwrap();
            self.state_bv.set(*index,false);
        }

        if last_same < self.usable_pieces.len() - 1 {
            self.push_to_queue(&last_same);
        }
        if first_same > 0
            && ! self.nth_bit_is_true(&first_same)
            && self.nth_bit_is_true(&(first_same-1))
        {
            self.push_to_queue(&(first_same-1));
        }
    }

    /// Finds what state the plank would be in if the `index`th usable
    /// piece were taken off the plank, and the `index+1`th usable piece
    /// were put on the plank, and adds that new state to the queue.
    fn push_to_queue(&mut self, index: &usize) {
        let mut new_bv = self.state_bv.clone();
        let mut remove_index = *index;
        let mut add_index = *index + 1;

        // find the leftmost piece ON the plank that equals the one we want
        // removed
        while remove_index > 0
            && self.nth_pieces_equal(&remove_index, &(remove_index-1))
            && self.nth_bit_is_true(&(remove_index-1))
        {
            remove_index -= 1;
        }
        // find the rightmost piece NOT on the plank that equals the one we
        // want added
        while add_index < self.usable_pieces.len() - 1
            && self.nth_pieces_equal(&add_index , &(add_index+1))
            && ! self.nth_bit_is_true(&(add_index+1))
        {
            add_index += 1;
        }

        new_bv.set(*self.usable_pieces.get(remove_index).unwrap(),false);
        new_bv.set(*self.usable_pieces.get(add_index).unwrap(),true);
        let new_total = self.total_len
            + self.nth_pieces_diff(&add_index,&remove_index).unwrap();
        self.bit_queue.push(new_bv,Reverse(NotNan::new(new_total).unwrap()));
    }
    pub fn has_next(&self) -> bool {
        !self.bit_queue.is_empty()
    }
    fn nth_bit_is_true(&self, index:&usize) -> bool {
        self.state_bv[*self.usable_pieces.get(*index).unwrap()]
    }
    fn nth_pieces_equal(&self, i_1:&usize, i_2:&usize) -> bool {
        let val_1 = self.usable_pieces.get(*i_1)
            .and_then(|x| self.all_pieces.get(*x));
        let val_2 = self.usable_pieces.get(*i_2)
            .and_then(|x| self.all_pieces.get(*x));
        match (val_1,val_2) {
            (Some(a),Some(b)) => *a == *b,
                            _ => false
        }
    }
    fn nth_pieces_diff(&self, i_1:&usize, i_2:&usize) -> Option<f64> {
        let val_1 = self.usable_pieces.get(*i_1)
            .and_then(|x| self.all_pieces.get(*x));
        let val_2 = self.usable_pieces.get(*i_2)
            .and_then(|x| self.all_pieces.get(*x));
        match (val_1,val_2) {
            (Some(a),Some(b)) => Some(*a - *b),
                            _ => None
        }
    }
    fn next(&mut self) {
        let (bv,wrapped_len) = self.bit_queue.pop().unwrap();
        self.total_len = match wrapped_len {
            Reverse(a) => a.into_inner()
        };
        self.state_bv = bv;

        for i in 0..self.usable_pieces.len() - 1 {
            if self.nth_bit_is_true(&i)
                && ! self.nth_bit_is_true(&(i+1))
            {
                self.push_to_queue(&i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plank_initializer() {
        let pieces = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut p = Plank::new(2,&bitvec![0;5],&pieces);
        assert_eq!(p.state_bv,bitvec![1,1,0,0,0]);
        assert_eq!(p.total_len, 3.0);

        p = Plank::new(3,&bitvec![0,1,0,1,0],&pieces);
        assert_eq!(p.state_bv,bitvec![1,1,1,1,1]);
        assert_eq!(p.total_len, 9.0);

        p.set_num_pieces(2);
        assert_eq!(p.state_bv,bitvec![1,1,1,1,0]);
        assert_eq!(p.total_len, 4.0);
    }

    #[test]
    fn test_next_fn_all_unique() {
        let pieces = vec![ 1.0, 3.0, 6.0, 7.0, 8.0,10.0,15.0,17.0,20.0];
        let mut p = Plank::new(3,&bitvec![0,1,0,0,0,1,0,1,0],&pieces);

        // 6 choose 3 = 20
        run_through_arrangements(&mut p,20);
    }
    #[test]
    fn test_next_some_repeats() {
        let pieces = vec![ 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0,
                           3.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0 ];
        let mut p = Plank::new(4,&bitvec![0,0,0,0, 0,0,1,1, 0,0,1,1, 0,0,0,0],
                               &pieces);
        run_through_arrangements(&mut p,27);
        p = Plank::new(4,&bitvec![0,0,1,1, 0,0,0,0, 0,0,0,0, 0,0,1,1],&pieces);
        run_through_arrangements(&mut p,27);
    }

    fn run_through_arrangements(p: &mut Plank, expected_count: usize) {
        let mut old_len = 0.0;
        let mut curr_len = p.total_len;
        let mut counter = 1;
        while p.has_next() {
            assert!(old_len <= curr_len);
            p.next();
            old_len = curr_len;
            curr_len = p.total_len;
            counter += 1;
        }
        assert_eq!(counter,expected_count);
    }
    #[test]
    fn test_soln_1() {
        let pieces = vec![ 0.2, 0.3, 0.45, 0.45, 0.45, 0.7, 0.85, 1.1, 1.5 ];
        let max_len = 2.0;
        let soln = find_solution(max_len, &pieces);
        for a in soln {
            let b = vec![ vec![ 0.2, 0.3, 1.5 ],
                          vec![ 0.45, 0.45, 1.1 ],
                          vec![ 0.45, 0.7, 0.85 ] ];
            assert!(b.iter().map(|c| c.iter().zip(a.iter())
                                 .fold(true, |x,(y,z)| x && (y == *z)))
                    .fold(false, |x,y| x||y));
        }
    }
}
