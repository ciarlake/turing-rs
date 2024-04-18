use std::collections::VecDeque;

#[derive(Debug)]
pub enum Move {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq)]
pub enum State<S> {
    State(S),
    Halt,
}

#[derive(Debug)]
pub struct Rule<S, Sym> {
    pub new_state: Option<State<S>>,
    pub write: Option<Sym>,
    pub head_move: Option<Move>,
}

pub trait Executor<S, Sym: Default> {
    fn execute(state: &S, symbol: &Sym) -> Rule<S, Sym>;
}

#[derive(Debug)]
pub struct Machine<S, Sym: Default> {
    state: State<S>,
    tape: VecDeque<Sym>,
    head: usize,
}

pub struct MachinePeek<'a, S, Sym: Default> {
    pub state: &'a State<S>,
    pub tape: (&'a [Sym], &'a [Sym]),
    pub head: usize,
}

impl<S, Sym> Machine<S, Sym>
where
    Sym: Default,
{
    pub fn new(state: S, tape: VecDeque<Sym>) -> Self {
        Self {
            state: State::State(state),
            tape,
            head: 0,
        }
    }

    pub fn execute<E>(&mut self)
    where
        E: Executor<S, Sym>,
    {
        let State::State(ref state) = self.state else {
            return;
        };

        let Rule {
            new_state,
            write,
            head_move,
        } = E::execute(state, self.tape.get(self.head).unwrap());

        if let Some(new_state) = new_state {
            self.state = new_state;
        }

        if let Some(write) = write {
            self.write_tape(write);
        }

        if let Some(head_move) = head_move {
            match head_move {
                Move::Left => self.head_move_left(),
                Move::Right => self.head_move_right(),
            }
        }
    }

    pub fn halted(&self) -> bool {
        matches!(&self.state, State::Halt)
    }

    pub fn peek(&self) -> MachinePeek<'_, S, Sym> {
        MachinePeek {
            state: &self.state,
            tape: self.tape.as_slices(),
            head: self.head,
        }
    }

    pub fn finish(self) -> (VecDeque<Sym>, State<S>) {
        (self.tape, self.state)
    }

    fn write_tape(&mut self, write: Sym) {
        *self.tape.get_mut(self.head).unwrap() = write;
    }

    fn head_move_left(&mut self) {
        match self.head {
            // if at the left end of tape expand the vec; don't change the index
            // to avoid underflow
            0 => self.tape.push_front(Sym::default()),
            // otherwise decrement head by one
            _ => self.head -= 1,
        }
    }

    fn head_move_right(&mut self) {
        if self.head == self.tape.len() - 1 {
            self.tape.push_back(Sym::default());
        }

        self.head += 1;
    }
}

impl<S, Sym> Default for Machine<S, Sym>
where
    S: Default,
    Sym: Default,
{
    fn default() -> Self {
        Self::new(S::default(), [Sym::default()].into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, PartialEq, Eq)]
    struct Inc;
    struct IncExecutor;

    impl Executor<Inc, bool> for IncExecutor {
        fn execute(state: &Inc, symbol: &bool) -> Rule<Inc, bool> {
            if *symbol {
                Rule {
                    new_state: None,
                    write: Some(false),
                    head_move: Some(Move::Right),
                }
            } else {
                Rule {
                    new_state: Some(State::Halt),
                    write: Some(true),
                    head_move: None,
                }
            }
        }
    }

    #[test]
    fn binary_inc_test() {
        let mut machine: Machine<Inc, bool> = Machine::new(Inc, [false, true].into());

        while !machine.halted() {
            machine.execute::<IncExecutor>();
        }

        let (mut vec, state) = machine.finish();

        vec.make_contiguous();
        let (vec_slice, _) = vec.as_slices();

        assert_eq!(vec_slice, &[true, true]);
        assert_eq!(state, State::Halt);

        let mut machine: Machine<Inc, bool> = Machine::new(Inc, [true, true].into());

        while !machine.halted() {
            machine.execute::<IncExecutor>();
        }

        let (mut vec, state) = machine.finish();

        vec.make_contiguous();
        let (vec_slice, _) = vec.as_slices();

        assert_eq!(vec_slice, &[false, false, true]);
        assert_eq!(state, State::Halt);
    }

    // TODO: Test something that involves traversing the head backwards
}
