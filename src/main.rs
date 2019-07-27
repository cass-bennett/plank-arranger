extern crate bit_vec;
use bit_vec::BitVec;
extern crate ordered_float;
use ordered_float::NotNan;

use priority_queue::PriorityQueue;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;

fn main() {
}

#[allow(dead_code)]
pub struct PlankArrangement<'a> {
    planks: Vec<Plank<'a>>,
    pieces: Vec<f64>,
    max_len: f64,
}

pub struct Plank<'a> {
    all_pieces: &'a Vec<f64>,
    usable_pieces: Vec<usize>,
    num_pieces: usize,
    rem_len: f64,
    state_bv: BitVec,
    bit_queue: PriorityQueue<(BitVec,usize),NotNan<f64>>,
}

impl<'a> Plank<'a> {
    pub fn new(n_p: usize,
               max_len: f64,
               prev_bits: &BitVec,
               pcs: &'a Vec<f64>, ) -> Plank<'a> {
        let mut available_pieces = Vec::new();
        let mut i = 0;
        while i < pcs.len() {
            if prev_bits.get(i) == Some(false) {
                available_pieces.push(i);
            }
            i += 1;
        }
        available_pieces.shrink_to_fit();
        let mut ret = Plank {
            all_pieces: pcs,
            usable_pieces: available_pieces,
            num_pieces: 0,
            rem_len: 0.0,
            state_bv: prev_bits.clone(),
            bit_queue: PriorityQueue::new(),
        };
        ret.set_num_pieces(n_p,max_len);
        return ret;
    }
    fn set_num_pieces(&mut self,
                      n_p: usize,
                      max_len: f64) {
        self.num_pieces = n_p;
        self.rem_len = max_len;
        self.bit_queue = PriorityQueue::new();

        for i in 0..self.num_pieces {
            let index = self.usable_pieces.get(i).unwrap();
            self.rem_len -= *self.all_pieces.get(*index).unwrap();
            self.state_bv.set(*index,true);
        }
        for i in self.num_pieces..self.usable_pieces.len() {
            let index = self.usable_pieces.get(i).unwrap();
            self.state_bv.set(*index,false);
        }
        if self.num_pieces > 0
            && self.num_pieces + 1 < self.usable_pieces.len()
        {
            self.push_to_queue( &(self.num_pieces-1) );
        }
    }
    fn push_to_queue(&mut self, index: &usize) {
        let bit_index = self.usable_pieces.get(*index).unwrap();
        let dest_index = self.usable_pieces.get(*index+1).unwrap();
        let future_len =
            self.rem_len
            - (*self.all_pieces.get(*dest_index).unwrap()
               - *self.all_pieces.get(*bit_index).unwrap());
        let wrapped_len = NotNan::new(future_len).unwrap();
        self.bit_queue.push((self.state_bv.clone(),*index),wrapped_len);
    }
    pub fn pieces_too_big(&self) -> bool {
        self.rem_len < 0.0
    }
    pub fn has_next(&self) -> bool {
        !self.bit_queue.is_empty()
    }
    fn nth_bit_is_true(&self, index:&usize) -> bool {
        self.state_bv[*self.usable_pieces.get(*index).unwrap()]
    }
    #[allow(dead_code)]
    fn nth_pieces_equal(&self, i_1:&usize, i_2:&usize) -> bool {
        match self.nth_pieces_diff(i_1,i_2) {
            Some(a) => a.abs() < 0.000000001,
            None    => false
        }
    }
    #[allow(dead_code)]
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
        let ((bv,index),wrapped_len) = self.bit_queue.pop().unwrap();
        self.rem_len = wrapped_len.into_inner();
        self.state_bv = bv;

        self.state_bv.set(*self.usable_pieces.get(index).unwrap(),false);
        self.state_bv.set(*self.usable_pieces.get(index+1).unwrap(),true);

        let queue_add_start = if index > 0 { index - 1 } else { 0 };

        for i in queue_add_start..self.usable_pieces.len()-1 {
            if self.nth_bit_is_true( &i )
                && ! self.nth_bit_is_true( &(i+1) )
            {
                self.push_to_queue( &i );
            }
        }
    }
}

#[allow(dead_code)]
fn get_pieces() -> (f64,Vec<f64>) {
    // get filename from the user
    print!("input file: ");
    io::stdout().flush().unwrap();
    let mut file_name = String::new();
    io::stdin().read_line(&mut file_name).unwrap();
    // remove trailing newline
    file_name.pop();

    // read contents of the file to a vector of floats
    let mut file = File::open(&file_name).expect("Couldn't open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Couldn't read the file");
    let mut v: Vec<f64> = contents.split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect::<Vec<_>>();
    
    // take the first number in the file as the plank length
    let plank_len = v.remove(0);
    // sort the numbers from low to high
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v.shrink_to_fit();

    return (plank_len,v);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plank_initializer() {
        let pieces = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut p = Plank::new(2,4.0,&BitVec::from_bytes(&[0x00]),&pieces);
        assert_eq!(p.state_bv,BitVec::from_bytes(&[0xc0]));
        assert_eq!(p.rem_len, 1.0);
        
        p = Plank::new(3,4.0,&BitVec::from_bytes(&[0x50]),&pieces);
        assert_eq!(p.state_bv,BitVec::from_bytes(&[0xf8]));
        assert_eq!(p.rem_len, -5.0);

        p.set_num_pieces(2,4.0);
        assert_eq!(p.state_bv,BitVec::from_bytes(&[0xf0]));
        assert_eq!(p.rem_len, 0.0);
    }

    #[test]
    fn test_next_function() {
        let pieces = vec![1.0, 3.0, 6.0, 7.0, 8.0, 10.0,15.0,17.0, 20.0];
        //  [0x45,0x0] =   0    1    0    0  |  0    1    0    1  |  0
        let mut p = Plank::new(3,43.0,&BitVec::from_bytes(&[0x45,0x0]),&pieces);
        let mut old_rem_len = 43.0;
        let mut curr_rem_len = p.rem_len;
        let mut counter = 1;
        while p.has_next() {
            assert!(curr_rem_len <= old_rem_len);
            p.next();
            old_rem_len = curr_rem_len;
            curr_rem_len = p.rem_len;
            counter += 1;
        }
        // 6 choose 3 = 20
        assert_eq!(counter,20);
    }
}
