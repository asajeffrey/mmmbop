use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::borrow::Borrow;

pub struct Node {
    internals: Rc<NodeInternals>,
}

struct NodeInternals {
    data: i64,
    next: RefCell<Option<Rc<NodeInternals>>>,
    prev: RefCell<Option<Rc<NodeInternals>>>,

    external_ref_count: Cell<usize>,
    next_node: RefCell<Option<Rc<NodeInternals>>>,
    prev_node: RefCell<Option<Rc<NodeInternals>>>,
    
}

impl Node {
    pub fn new(data: i64) -> Node {
        println!("Node {} has external count 1 (init).", data);
        Node {
            internals: Rc::new(NodeInternals {
                data: data,
                next: RefCell::new(None),
                prev: RefCell::new(None),
                external_ref_count: Cell::new(1),
                next_node: RefCell::new(None),
                prev_node: RefCell::new(None),
            })
        }
    }
    pub fn connect(&self, other: &Node) {
        if let Some(old_next) = self.internals.next.borrow_mut().take() {
            if let Some(ref old_next_node) = *self.internals.next_node.borrow() {
                *old_next_node.prev_node.borrow_mut() = None;
                *old_next.prev.borrow_mut() = None;
            } else {
                reclaim(old_next);
            }
        }
        if let Some(old_prev) = other.internals.prev.borrow_mut().take() {
            if let Some(ref old_prev_node) = *other.internals.prev_node.borrow() {
                *old_prev_node.next_node.borrow_mut() = None;
                *old_prev.next.borrow_mut() = None;
            } else {
                reclaim(old_prev);
            }
        }
        *self.internals.next.borrow_mut() = Some(other.internals.clone());
        *other.internals.prev.borrow_mut() = Some(self.internals.clone());
        *self.internals.next_node.borrow_mut() = Some(other.internals.clone());
        *other.internals.prev_node.borrow_mut() = Some(self.internals.clone());
    }
    pub fn data(&self) -> i64 {
        self.internals.data
    }
}

impl Clone for Node {
    fn clone(&self) -> Node {
        from_internals(self.internals.clone())
    }
}

fn from_internals(internals: Rc<NodeInternals>) -> Node {
    let count = internals.external_ref_count.get() + 1;
    internals.external_ref_count.set(count);
    println!("Node {} has external count {} (incr).", internals.data, count);
    Node{ internals: internals }
}

impl Drop for Node {
    fn drop(&mut self) {
        let count = self.internals.external_ref_count.get().wrapping_sub(1);
        self.internals.external_ref_count.set(count);
        println!("Node {} has external count {} (decr).", self.internals.data, count);
        if self.internals.external_ref_count.get() == 0 {
            if let Some(ref prev_node) = *self.internals.prev_node.borrow() {
                *prev_node.next_node.borrow_mut() = self.internals.next_node.borrow().clone();
            }
            if let Some(ref next_node) = *self.internals.next_node.borrow() {
                *next_node.prev_node.borrow_mut() = self.internals.prev_node.borrow().clone();
            }
            if self.internals.next_node.borrow().is_none() && self.internals.prev_node.borrow().is_none() {
                reclaim(self.internals.clone());
            }
        }
    }
}

fn reclaim(internals: Rc<NodeInternals>) {
    println!("Reclaiming {}.", internals.data);
    let mut next = internals.next.borrow_mut().take();
    while let Some(curr) = next {
        println!("Nulling {}.", curr.data);
        curr.prev.borrow_mut().take();
        next = curr.next.borrow_mut().take();
    }
    let mut prev = Some(internals);
    while let Some(curr) = prev {
        println!("Nulling {}.", curr.data);
        prev = curr.prev.borrow_mut().take();
        curr.next.borrow_mut().take();
    }
}

#[test]
fn testy() {
    let n1 = Node::new(1);
    {
        let n2 = Node::new(2);
        let n3 = Node::new(3);
        let n4 = Node::new(4);
        n1.connect(&n2);
        n2.connect(&n3);
        // After doing this, n2 and n3 can be dropped
        n1.connect(&n4);
        println!("{}.", n1.data());
    }
    assert_eq!(1, n1.data());
}
