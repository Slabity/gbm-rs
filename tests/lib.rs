extern crate gbm;

#[test]
fn user_data() {
    let file = std::fs::OpenOptions::new().read(true).write(true).open("/dev/dri/card0").unwrap();
    let dev = gbm::Device::from_file(&file).unwrap();
    let format = gbm::Format::XRGB8888;
    let flags = gbm::SCANOUT | gbm::RENDERING;
    let buffer = dev.buffer((16, 16), format, flags).unwrap();

    // Get the user data. It should be None since we haven't set it.
    let get = unsafe { buffer.get_user_data::<u32>() };
    assert_eq!(get, None);

    {
        // Set the user data to an Rc of 12345
        let set = std::rc::Rc::new(12345);
        buffer.set_user_data(Some(set.clone()));

        // Drop the original Rc. The buffer should still have one.
    }

    let weak = {
        // Getting the data should return an Rc of 12345
        let get = unsafe { buffer.get_user_data::<u32>().unwrap() };
        assert_eq!(*get, 12345);

        // Set the new user data to nothing.
        buffer.set_user_data::<()>(None);

        // Get a weak reference and drop the last Rc
        std::rc::Rc::downgrade(&get)
    };

    // With all counts dropped, weak should return None
    assert_eq!(weak.upgrade(), None);

    // User data should now be none.
    let get = unsafe { buffer.get_user_data::<()>() };
    assert_eq!(get, None);
}

