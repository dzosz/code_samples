use std::mem;

struct Node {
    val : i32,
    left : Option<Box<Node>>,
    right : Option<Box<Node>>
}

impl Node {
    fn insert(&mut self, val : i32) {
        if self.val == val {
            return;
        }

        if val < self.val {
            if self.left.is_none() {
                self.left = Some(Box::new(Node { val : val, left: None, right: None}));
            }
            self.left.as_mut().unwrap().insert(val);
        } else {
            if self.right.is_none() {
                self.right = Some(Box::new(Node { val : val, left: None, right: None}));
            }
            self.right.as_mut().unwrap().insert(val);
        }
    }

    fn mirror(&mut self) {
        match self.left {
            Some(ref mut sub) => sub.mirror(),
            _ => {},
        }
        match self.right {
            Some(ref mut sub) => sub.mirror(),
            _ => {},
        }
        mem::swap(&mut self.left, &mut self.right);
    }

    fn debug(&self, str : String) {
        match self.left {
            Some(ref sub) => { sub.debug(str.clone() + &format!("->L {}", sub.val)); },
            _ => {println!("{}", str);},
        }
        match self.right {
            Some(ref sub) => { sub.debug(str.clone() + &format!("->R {}", sub.val)); },
            _ => {println!("{}", str);},
        }
    }
}

fn main() {

    let mut head = Node { val : 50, left: None, right: None };
    head.insert(50);
    head.insert(25);
    head.insert(60);

    head.insert(26);
    head.insert(28);

    head.insert(58);
    head.insert(71);

    let mut start = format!("{} ", head.val);
    head.debug(start.clone());
    head.mirror();
    println!("");
    head.debug(start.clone());
}
