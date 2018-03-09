extern crate permutohedron;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
struct Box2d {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32
}

impl Box2d {
    fn contained_by(&self, other: &Box2d) -> bool {
        self.x1 >= other.x1 &&
            self.x2 <= other.x2 &&
            self.y1 >= other.y1 &&
            self.y2 <= other.y2
    }
    fn intersects(&self, other: &Box2d) -> bool {
        self.x1 < other.x2 &&
            self.x2 > other.x1 &&
            self.y1 < other.y2 &&
            self.y2 > other.y1
    }


    fn partially_overlaps(&self, other: &Box2d) -> bool {
        self.intersects(other) && !self.contained_by(other)
    }/*
    fn union(&self, other: &Box2d) -> Box2d {
        Box2d {
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
            x2: self.x2.max(other.x2),
            y2: self.y2.max(other.y2),
        }
    }*/
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
struct Box2dOrEmpty {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32
}

impl Box2dOrEmpty {
    fn union(&self, other: &Box2d) -> Box2dOrEmpty {
        if self.x1 == self.x2 {
            Box2dOrEmpty {
                x1: (other.x1),
                y1: (other.y1),
                x2: (other.x2),
                y2: (other.y2),
            }
        } else {
            Box2dOrEmpty {
                x1: self.x1.min(other.x1),
                y1: self.y1.min(other.y1),
                x2: self.x2.max(other.x2),
                y2: self.y2.max(other.y2),
            }
        }
    }
    fn intersects(&self, other: &Box2d) -> bool {
        if self.x1 == self.x2 { return false }
        self.x1 < other.x2 &&
            self.x2 > other.x1 &&
            self.y1 < other.y2 &&
            self.y2 > other.y1
    }
    fn unwrap(&self) -> Box2d {
        assert_ne!(self.x1, self.x2);
        Box2d { x1: self.x1, x2: self.x2, y1: self.y1, y2: self.y2 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Shape {
    bounds: Box2d,
    id: char
}

struct Dag {
    parents: Vec<Vec<usize>>
}


fn build_dag(list: &[Shape]) -> Dag {
    let mut d = Dag {parents: Vec::new()};
    for i in 0..list.len() {
        let mut parents = Vec::new();
        for j in i+1..list.len() {
            if list[j].bounds.intersects(&list[i].bounds) {
                parents.push(j);
            }
        }
        d.parents.push(parents);
    }
    d
}

fn diff(old: &[Shape], new: &[Shape]) -> (Box2d, Vec<Shape>) {
    let mut dirty = Box2dOrEmpty{ x1: 0, x2: 0, y1: 0, y2: 0};

    for i in old {
        if !new.contains(i) {
            dirty = dirty.union(&i.bounds);
        }
    }
    for i in new {
        if !old.contains(i) {
            dirty = dirty.union(&i.bounds);
        }
    }

    let mut result = Vec::new();
    for i in new {
        if dirty.intersects(&i.bounds) {
            result.push(*i);
        }
    }
    (dirty.unwrap(), result)
}

fn p(x: &[Shape]) {
    println!("{:?}", x.iter().map(|x| x.id).collect::<Vec<_>>());
}

fn equiv(a: &[Shape], b: &[Shape]) -> bool {
    //println!("{:?}\n{:?}", a, b);
    let ad = build_dag(a);
    let bd = build_dag(b);
    for i in 0..a.len() {
        let j = b.iter().position(|x| *x == a[i]).unwrap();
        if ad.parents[i].len() != bd.parents[j].len() {
            println!("bad length");
            return false
        }
        for sa in ad.parents[i].iter().map(|x| a[*x]) {
            let mut found = false;
            for sb in bd.parents[j].iter().map(|x| b[*x]) {
                if sa == sb {
                    found = true;
                    break;
                }
            }
            if !found {
                print_graph(a);
                print_graph(b);
                return false
            }
        }
    }
    true
}

fn find(hay: &[Shape], needle: Shape) -> Option<usize> {
    hay.iter().position(|x| *x == needle)
}

fn check_ordering(src: &[Shape], target: &[Shape]) -> bool {
    let od = build_dag(src);
    for i in 0..src.len() {
        if let Some(ipos) = find(target, src[i]) {
            for p in &od.parents[i] {
                if let Some(ppos) = find(target, src[*p]) {
                    if ipos > ppos {
                        println!("{} should be before {}", target[ipos].id, target[ppos].id);
                        return false
                    }
                }
            }
        }
    }
    true

}

fn check_merge(old: &[Shape], new: &[Shape], result: &[Shape]) -> bool {
    check_ordering(old, result) &&
    check_ordering(new, result)
}

fn bogo_merge(old: &[Shape], new: &[Shape], dirty: Box2d) -> Vec<Shape> {
    let mut result = merge_bad(old, new, dirty);
    // take a badly merged result and permute it until passes check_merge
    {
        let perm = permutohedron::Heap::new(&mut result);
        for data in perm {
            if check_merge(old, new, &data) {
                return data
            }
        }
    }
    return result
}

fn merge_bad(old: &[Shape], new: &[Shape], dirty: Box2d) -> Vec<Shape> {

    //println!("dirty: {:?}", dirty);
    let mut result = Vec::new();
    let mut oi = old.iter();
    for n in new {
        if n.bounds.partially_overlaps(&dirty) {
            while let Some(o) = oi.next() {
                if n.bounds == o.bounds {
                    break;
                } else if o.bounds.intersects(&dirty) {

                } else {
                    result.push(*o);
                }
            }
        }
        result.push(*n)
    }
    while let Some(o) = oi.next() {
        if o.bounds.intersects(&dirty) {

        } else {
            result.push(*o)
        }
    }
    result
}

fn print_graph(list: &[Shape]) {
    let d = build_dag(&list);

    println!("{{");
    for i in 0..list.len() {
        println!("  {} <- {:?}", list[i].id,
                 d.parents[i].iter().map(|x| list[*x].id)
                     .collect::<Vec<_>>());
    }
    println!("}}");
}

fn do_merge(s1: &[Shape], s2: &[Shape], s3: &[Shape]) {

    let d1 = diff(&s1, &s2);
    let d2 = diff(&s2, &s3);

    let r1 = s1.clone();
    let r2 = bogo_merge(&r1, &d1.1, d1.0);
    let r3 = bogo_merge(&r2, &d2.1, d2.0);

    //p(&diff(&s1, &s2).1);
    //p(&diff(&s2, &s3).1);

    //p(&r1);
    //p(&r2);
    //print_graph(&r2);
    //print_graph(&d2.1);
    //p(&r3);
    assert!(check_merge(&r2, &d2.1, &r3));
    assert!(equiv(&r3, &s3))
}

fn select(s: i32, list: &[Shape]) -> Vec<Shape> {
    let mut result = Vec::new();
    for i in 0..4 {
        if s & (1 << i) != 0 {
            result.push(list[i])
        }
    }
    result
}
#[allow(non_snake_case)]
fn do_all_merges() {
    let A = Shape {id: 'A', bounds: Box2d { x1: 250, y1: 50, x2: 350, y2: 150 }};
    let B = Shape {id: 'B', bounds: Box2d { x1: 200, y1: 0, x2: 300, y2: 100 }};
    let C = Shape {id: 'C', bounds: Box2d { x1: 0, y1: 0, x2: 100, y2: 100 }};
    let D = Shape {id: 'D', bounds: Box2d { x1: 80, y1: 20, x2: 220, y2: 120 }};

    let mut list = vec![A, B, C, D];

    let perm = permutohedron::Heap::new(&mut list);

    for d in perm {
        for is1 in 0..16 {
            for is2 in 0..16 {
                for is3 in 0..16 {
                    if is1 != is2 && is2 != is3 {
                        let s1 = select(is1, &d);
                        let s2 = select(is2, &d);
                        let s3 = select(is3, &d);
                        p(&s1);
                        p(&s2);
                        p(&s3);
                        do_merge(&s1, &s2, &s3);
                        println!("");
                    }
                }
            }
        }
    }

}
#[allow(non_snake_case)]
fn main() {

    let A = Shape {id: 'A', bounds: Box2d { x1: 250, y1: 50, x2: 350, y2: 150 }};
    let B = Shape {id: 'B', bounds: Box2d { x1: 200, y1: 0, x2: 300, y2: 100 }};
    let C = Shape {id: 'C', bounds: Box2d { x1: 0, y1: 0, x2: 100, y2: 100 }};
    let D = Shape {id: 'D', bounds: Box2d { x1: 80, y1: 20, x2: 220, y2: 120 }};

    //let list = vec![A, B, C, D];
    //print_graph(&list);
    do_merge(&vec![], &vec![B,A], &vec![B,A,D]);

    do_merge(&vec![C, D], &vec![A,B,C,D], &vec![A,B,C]);
    do_merge(&vec![B, A, D], &vec![C,B,A,D], &vec![C,B,A]);
    do_merge(&vec![A, B, D], &vec![A,B,C,D], &vec![A,B,C]);
    do_merge(&vec![A], &vec![A, D, C], &vec![A, B, D, C]);

    do_all_merges();
    //let s2_res = vec![C, D, A];
    //println!("eq; {}", equiv(&s2_res, &s2));

}
