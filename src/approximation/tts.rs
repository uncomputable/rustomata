use num_traits::Zero;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{AddAssign, MulAssign};

use approximation::*;
use push_down_automaton::*;
use tree_stack_automaton::*;

/// `ApproximationStrategy` that approximates a `TreeStackAutomaton` into a `PushDownAutomaton`
#[derive(Clone, Debug)]
pub struct TTSElement<A> {
    _dummy: PhantomData<A>,
}

impl<A> TTSElement<A> {
    pub fn new() -> Self {
        TTSElement {
            _dummy: PhantomData,
        }
    }
}

impl<A, T, W> ApproximationStrategy<T, W> for TTSElement<A>
    where A: Clone + Hash + Ord,
          T: Clone + Eq + Hash + Ord,
          W: AddAssign + Copy + MulAssign + One + Ord + Zero,
{
    type I1 = TreeStackInstruction<A>;
    type I2 = PushDownInstruction<A>;
    type A1 = TreeStackAutomaton<A, T, W>;
    type A2 = PushDownAutomaton<A, T, W>;

    fn approximate_storage(&self, mut ts: TreeStack<A>)-> PushDown<A> {
        let mut pd = Vec::new();
        pd.push(ts.current_symbol().clone());

        while let Ok(smaller) = ts.down() {
            pd.push(smaller.current_symbol().clone());
            ts = smaller;
        }

        pd.reverse();
        PushDown::from_vec(pd)
    }

    fn approximate_instruction(&self, instr: &TreeStackInstruction<A>)
                               -> PushDownInstruction<A>
    {
        match *instr {
            TreeStackInstruction::Up { ref current_val, ref new_val, ..}
            | TreeStackInstruction::Push { ref current_val, ref new_val, .. } => {
                PushDownInstruction::Replace {
                    current_val: vec![current_val.clone()],
                    new_val: vec![current_val.clone(), new_val.clone()],
                }
            },
            TreeStackInstruction::Down { ref current_val, ref old_val, ref new_val } => {
                PushDownInstruction::Replace {
                    current_val: vec![current_val.clone(), old_val.clone()],
                    new_val: vec![new_val.clone()],
                }
            },
        }
    }

}
