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
