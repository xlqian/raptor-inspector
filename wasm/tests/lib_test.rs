use wasm::echo;

#[test]
fn test_public_echo() {
    assert_eq!(echo("hello, wasm!"), "hello, wasm!");
}
