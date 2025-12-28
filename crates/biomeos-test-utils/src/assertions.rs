//! Test Assertions
//!
//! Custom assertions for BiomeOS testing that provide better error messages.

/// Assert that a Result is Ok and return the value
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
    ($result:expr, $($arg:tt)+) => {
        match $result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {:?}. {}", e, format!($($arg)+)),
        }
    };
}

/// Assert that a Result is Err
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(val) => panic!("Expected Err, got Ok: {:?}", val),
            Err(_) => (),
        }
    };
    ($result:expr, $($arg:tt)+) => {
        match $result {
            Ok(val) => panic!("Expected Err, got Ok: {:?}. {}", val, format!($($arg)+)),
            Err(_) => (),
        }
    };
}

/// Assert that an Option is Some and return the value
#[macro_export]
macro_rules! assert_some {
    ($option:expr) => {
        match $option {
            Some(val) => val,
            None => panic!("Expected Some, got None"),
        }
    };
    ($option:expr, $($arg:tt)+) => {
        match $option {
            Some(val) => val,
            None => panic!("Expected Some, got None. {}", format!($($arg)+)),
        }
    };
}

/// Assert that an Option is None
#[macro_export]
macro_rules! assert_none {
    ($option:expr) => {
        match $option {
            Some(val) => panic!("Expected None, got Some: {:?}", val),
            None => (),
        }
    };
    ($option:expr, $($arg:tt)+) => {
        match $option {
            Some(val) => panic!("Expected None, got Some: {:?}. {}", val, format!($($arg)+)),
            None => (),
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_assert_ok() {
        let result: Result<i32, &str> = Ok(42);
        let value = assert_ok!(result);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_assert_err() {
        let result: Result<i32, &str> = Err("error");
        assert_err!(result);
    }

    #[test]
    fn test_assert_some() {
        let option = Some(42);
        let value = assert_some!(option);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_assert_none() {
        let option: Option<i32> = None;
        assert_none!(option);
    }
}

