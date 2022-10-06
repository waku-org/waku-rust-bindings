mod events;
mod general;
mod node;

#[cfg(test)]
mod tests {
    use std::ffi::CStr;
    use std::os::raw::c_char;
    use waku_sys::waku_content_topic;

    #[test]
    fn content_topic() {
        let topic = unsafe {
            waku_content_topic(
                "foo_bar".as_ptr() as *mut c_char,
                1,
                "foo_topic".as_ptr() as *mut c_char,
                "rfc26".as_ptr() as *mut c_char,
            )
        };

        let topic_str = unsafe { CStr::from_ptr(topic) }
            .to_str()
            .expect("Decoded return");
        println!("{}", topic_str);
    }
}
