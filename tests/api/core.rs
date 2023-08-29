// use consoxide::io::tmux::{
//     pane::{InSession, Pane},
//     session::TmuxSession,
// };
//
// #[test]
// fn tmux_io_monitor_works() {
//     dotenv::dotenv().ok();
//     let pane = Pane::new(&std::env::var("WATCHED_PANE").expect("Problem with env"));
//
//     let out = pane.run_input("echo Test".to_string());
//
//     // let last_out = pane.last_io();
//     assert_eq!(out, "Test\n");
// }
//
// #[test]
// fn test_to_out() {
//     let session = TmuxSession::new();
//     session.to_out("Im going to be so annoyed if i can't see this entire message. I've worked so hard and yet I find myself having to deal with another dumbass problem.");
// }
