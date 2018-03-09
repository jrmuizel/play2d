extern crate permutohedron;


macro_rules! dlog {
    //($($e:expr),*) => { {$(let _ = $e;)*} }
    ($($t:tt)*) => { println!($($t)*) }
}

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
    fn new() -> Box2dOrEmpty {
        Box2dOrEmpty{ x1: 0, x2: 0, y1: 0, y2: 0}
    }
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
    fn empty(&self) -> bool {
        self.x1 == self.x2
    }
    // always returns false if empty
    fn intersects(&self, other: &Box2d) -> bool {
        if self.empty() { return false }
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
    let mut dirty = Box2dOrEmpty::new();

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

fn ids(x: &[Shape]) -> Vec<char> {
    x.iter().map(|x| x.id).collect::<Vec<_>>()
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

// makes sure that 'target' obeys all of the ordering constraints imposed by 'src'
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

#[allow(dead_code)]
fn bogo_merge_v1(old: &[Shape], new: &[Shape], dirty: Box2d) -> Vec<Shape> {
    let mut result = merge_bad(old, new, dirty);
    // take a badly merged result and permutes it until passes check_merge
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

fn bogo_merge(old: &[Shape], new: &[Shape], dirty: Box2d) -> Vec<Shape> {
    let result = merge_good(old, new, dirty);
    if let Some(result) = result {
        println!("good");
        result
    } else {
        println!("bad");
        let mut result = merge_bad(old, new, dirty);
        // take a badly merged result and permutes it until passes check_merge
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
}

fn merge_good_v1(old: &[Shape], new: &[Shape], dirty: Box2d) -> Option<Vec<Shape>> {
    // as long as we're not swapping the order items we don't need to worry
    // about intersections
    let mut result = Vec::new();
    let mut oi = old.iter();
    for n in new {
        if n.bounds.partially_overlaps(&dirty) {
            while let Some(o) = oi.next() {
                if n.bounds == o.bounds {
                    break;
                } else if o.bounds.contained_by(&dirty) {
                    // we can drop these items
                } else if o.bounds.partially_overlaps(&dirty) {
                    return None
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
    Some(result)
}

fn merge_good_v2(old: &[Shape], new: &[Shape], dirty: Box2d) -> Option<Vec<Shape>> {
    // as long as we're not swapping the order items we don't need to worry
    // about intersections
    let mut result = Vec::new();
    let mut oi = old.iter();
    for n in new {
        if n.bounds.partially_overlaps(&dirty) {
            while let Some(o) = oi.next() {
                if n.bounds == o.bounds {
                    break;
                } else if o.bounds.contained_by(&dirty) {
                    // we can drop these items
                } else if o.bounds.partially_overlaps(&dirty) {
                    // find the items that overlap this item
                    // those will need to move too
                    let mut oii = oi.clone();
                    while let Some(oo) = oii.next() {
                        if o.bounds.intersects(&oo.bounds) {
                            return None;
                        }
                    }
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
    Some(result)
}



fn merge_good_index(old: &[Shape], new: &[Shape], dirty: Box2d) -> Option<Vec<Shape>> {
    // as long as we're not swapping the order items we don't need to worry
    // about intersections
    let mut result = Vec::new();
    let mut oi = 0;
    dlog!("begin merge");
    for ni in 0..new.len() {
        let n = new[ni];
        //println!("{}", n.id);
        if n.bounds.partially_overlaps(&dirty) {
            dlog!("{} partially overlaps", n.id);
            while oi < old.len() {
                let o = old[oi];
                dlog!("old {}", o.id);
                oi+=1;
                if n.bounds == o.bounds {
                    break;
                } else if o.bounds.contained_by(&dirty) {
                    // we can drop these items
                } else if o.bounds.partially_overlaps(&dirty) {
                    dlog!("old partially {}", o.id);
                    // find the items that overlap this item
                    // those will need to move too
                    for oii in oi..old.len() {
                        let oo = old[oii];
                        if o.bounds.intersects(&oo.bounds) {
                            dlog!("intersec");
                            let odep = oii;
                            for nii in ni..new.len() {

                            }
                        }
                    }
                } else {
                    result.push(o);
                }
            }
        }
        result.push(n)
    }
    while oi < old.len() {
        let o = old[oi];
        if o.bounds.intersects(&dirty) {

        } else {
            result.push(o)
        }
        oi+=1;
    }
    Some(result)
}

fn merge_good(old: &[Shape], new: &[Shape], dirty: Box2d) -> Option<Vec<Shape>> {
    let mut result = Vec::new();
    let mut defer: Vec<Shape> = Vec::new();
    let mut oi = old.iter();
    dlog!("new {:?}, old {:?}", ids(new), ids(old));
    for n in new {
        dlog!("new {}", n.id);
        if n.bounds.partially_overlaps(&dirty) {
            dlog!("{} partially overlaps", n.id);
            if let Some(&d) = defer.get(0) {
                if d.bounds == n.bounds {
                    defer.remove(0);
                }
            } else {
                while let Some(o) = oi.next() {
                    if n.bounds == o.bounds {
                        break;
                    } else if o.bounds.contained_by(&dirty) {
                        // we can drop these items
                    } else if o.bounds.partially_overlaps(&dirty) {
                        dlog!("defer {}", o.id);
                        defer.push(*o);
                    } else {
                        if let Some(&d) = defer.get(0) {
                            if o.bounds.intersects(&d.bounds) {
                                dlog!("defer {}", o.id);
                                defer.push(*o);
                            } else {
                                result.push(*o);
                            }
                        } else {
                            result.push(*o);
                        }
                    }
                }
            }
        }
        if let Some(&d) = defer.get(0) {
            if d.bounds == n.bounds {
                defer.remove(0);
            }
        }
        result.push(*n);
    }
    dlog!("defer {:?}, result: {:?}", ids(&defer), ids(&result));
    result.append(&mut defer);

    while let Some(o) = oi.next() {
        if o.bounds.intersects(&dirty) {

        } else {
            if defer.len() > 0 { panic!(); }
            result.push(*o);
/*
            if let Some(&d) = defer.get(0) {
                if o.bounds.intersects(&d.bounds) {
                    defer.push(*o);
                } else {
                    result.push(*o);
                }
            } else {
                result.push(*o);
            }*/
        }
    }

    Some(result)
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
    //p(&r1);

    println!("diff");
    p(&d1.1);

    let r2 = bogo_merge(&r1, &d1.1, d1.0);
    p(&r2);
    println!("diff");

    p(&d2.1);
    let r3 = bogo_merge(&r2, &d2.1, d2.0);

    //print_graph(&r2);
    //print_graph(&d2.1);
    p(&r3);
    assert!(check_merge(&r2, &d2.1, &r3));
    assert!(equiv(&r3, &s3))
}

fn choose(s: i32, list: &[Shape]) -> Vec<Shape> {
    let mut result = Vec::new();
    for i in 0..list.len() {
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
    let E = Shape {id: 'E', bounds: Box2d { x1: 81, y1: 20, x2: 220, y2: 120 }};


    let mut list = vec![A, B, C, D, E];
    let combination_max = 1<<list.len();


    let perm = permutohedron::Heap::new(&mut list);


    for d in perm {
        for is1 in 0..combination_max {
            for is2 in 0..combination_max {
                for is3 in 0..combination_max {
                    // make sure there are differences between the states
                    if is1 != is2 && is2 != is3 {
                        let s1 = choose(is1, &d);
                        let s2 = choose(is2, &d);
                        let s3 = choose(is3, &d);
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
    let E = Shape {id: 'E', bounds: Box2d { x1: 81, y1: 20, x2: 220, y2: 120 }};


    //do_merge(&vec![A], &vec![A, D, C], &vec![A, B, D, C]); // ms4
    //do_merge(&vec![C, D], &vec![C, B, A, D], &vec![C, B, A, D, E]); // first broke 5
    do_merge(&vec![A], &vec![A, D, C, E], &vec![A, D, B, C, E]); // first broke 5


    //return;
    //let list = vec![A, B, C, D];
    //print_graph(&list);
    //do_merge(&vec![], &vec![B,A], &vec![B,A,D]);
    if true {
        do_merge(&vec![A], &vec![A, C, D], &vec![A, B, D]);


        do_merge(&vec![C, D], &vec![A, B, C, D], &vec![A, B, C]);


        do_merge(&vec![B, A, D], &vec![C, B, A, D], &vec![C, B, A]);
        do_merge(&vec![A, B, D], &vec![A, B, C, D], &vec![A, B, C]);
        do_merge(&vec![A], &vec![A, D, C], &vec![A, B, D, C]); // ms4
        //return;

    }
    do_merge(&vec![A, B, D], &vec![A, B, C, D], &vec![A, B, C]);

    //return;
    do_all_merges();

}