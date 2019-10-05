extern crate ring_buffer;
use ring_buffer::RingBuffer;

#[test]
fn test1() {
    let rb: RingBuffer = RingBuffer::new(16);
    let buf: [u8; 17] = [1; 17];
    assert_eq!(rb.write(&buf), 16);

    let buf: [u8; 17] = [2; 17];
    assert_eq!(rb.write(&buf), 0);
}

#[test]
fn test2() {
    let rb = RingBuffer::new(16);
    let buf: [u8; 8] = [2; 8];
    rb.write(&buf);
    assert_eq!(rb.len(), 8);

    let buf: [u8; 9] = [3; 9];
    rb.write(&buf);
    assert_eq!(rb.len(), 16);
}

#[test]
fn test3() {
    let rb = RingBuffer::new(16);
    let buf: [u8; 8] = [3; 8];
    rb.write(&buf);
    assert_eq!(rb.len(), 8);

    let read_buf = rb.read_all();
    assert_eq!(read_buf.len(), 8);
}

#[test]
fn test_4() {
    let rb = RingBuffer::new(16);
    let buf: [u8; 8] = [2; 8];
    rb.write(&buf);
    assert_eq!(rb.read_all(), vec![2; 8]);

    let buf: [u8; 8] = [3; 8];
    rb.write(&buf);
    assert_eq!(rb.read_all(), vec![3; 8]);
}

#[test]
fn test_5() {
    let rb = RingBuffer::new(16);
    let buf: [u8; 9] = [2; 9];
    assert_eq!(rb.write(&buf), 9);

    let buf: [u8; 9] = [3; 9];
    assert_eq!(rb.write(&buf), 7);
    assert_eq!(
        rb.read_all(),
        vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3]
    );
}

#[test]
fn test_6() {
    let mut rb = RingBuffer::new(16);
    let buf: [u8; 32] = [2; 32];

    rb.set_callback(|result: Vec<u8>| {
        assert_eq!(result, vec![2; 16]);
    });

    rb.callback_by_write(&buf).unwrap();
}

#[test]
fn test_7() {
    use std::sync::Arc;
    use std::thread::{sleep_ms, spawn};

    let rb = Arc::new(RingBuffer::new(16));

    let tmp = rb.clone();
    let j1 = spawn(move || {
        let buffer = [2; 9];
        assert_eq!(tmp.write(&buffer), 9);
    });

    let tmp2 = rb.clone();
    let j2 = spawn(move || {
        sleep_ms(100);
        let buffer = [1; 9];
        assert_eq!(tmp2.write(&buffer), 7);
    });

    j1.join().unwrap();
    j2.join().unwrap();

    assert_eq!(
        rb.read_all(),
        vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1]
    );
}

#[test]
fn test_8() {
    use std::sync::Arc;
    use std::thread::spawn;

    let rb = Arc::new(RingBuffer::new(16));

    let tmp1 = rb.clone();
    let tmp2 = rb.clone();

    let j1 = spawn(move || {
        let buffer = [1; 9];
        assert_eq!(tmp1.write(&buffer), 9);
    });

    let j2 = spawn(move || {
        loop {
            let result = tmp2.read_all();
            if result.len() > 0 {
                assert_eq!(result.len(), 9);
                assert_eq!(result, vec![1; 9]);
                break;
            }
        }
    });

    j1.join().unwrap();
    j2.join().unwrap();
}

#[test]
fn test_9() {
    use std::sync::Arc;
    use std::thread::{spawn, sleep_ms};

    let rb = Arc::new(RingBuffer::new(16));
    
    let tmp1 = rb.clone();
    let tmp2 = rb.clone();
    let tmp3 = rb.clone();

    let j1 = spawn(move || {
        assert_eq!(tmp1.write(&[1; 9]), 9);
    });

    let j2 = spawn(move || {
        assert_eq!(tmp2.write(&[2; 9]), 7);
    });

    let j3 = spawn(move || {
        let result = tmp3.read_all();
        assert!(result == vec![1,1,1,1,1,1,1,1,1,2,2,2,2,2,2,2] || result == vec![2,2,2,2,2,2,2,1,1,1,1,1,1,1,1,1] || result.is_empty());
    });

    j1.join();
    j2.join();
    j3.join();
}