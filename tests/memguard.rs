#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]
extern crate conveyor;

use conveyor::memguard::Partition;

describe! partition {
    before_each {

    }
    it "create and delete 5 partitions" {
        let partitions: Vec<Partition> = (0..5).map(|| {
            Partition::new()
        }).collect::<Vec<Partition>>();

        assert!((0..5).all(|n| partitions[n].id() == (n + 1) * 4))
    }
}