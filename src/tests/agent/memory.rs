use crate::lib::agent::config::memory::Memory;

#[test]
fn short_term_switch_works() {
    let mut context = Memory::ShortTerm.init();
    context.append_to_messages("tester", "test");
    context.append_to_messages("tester", "test2");
    let old = context.messages.clone();
    context.switch(Memory::Temporary);
    context.append_to_messages("system", "");
    context.switch(Memory::ShortTerm);
    assert_eq!(context.messages, old)
}
