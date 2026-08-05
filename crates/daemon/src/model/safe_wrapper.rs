/// Safe wrapper around the FFI boundary for model inference.
///
/// All native handle access goes through this module so the scheduler
/// never touches unsafe code directly.
pub fn validate_model_handle_ready(loaded: bool) -> bool {
    loaded
}

pub fn check_null_pointer<T>(ptr: *const T) -> bool {
    !ptr.is_null()
}

pub fn validate_buffer_size(size: usize, max: usize) -> bool {
    size > 0 && size <= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_ready_true() {
        assert!(validate_model_handle_ready(true));
    }

    #[test]
    fn test_handle_ready_false() {
        assert!(!validate_model_handle_ready(false));
    }

    #[test]
    fn test_null_pointer_rejected() {
        let ptr: *const u8 = std::ptr::null();
        assert!(!check_null_pointer(ptr));
    }

    #[test]
    fn test_valid_pointer_accepted() {
        let x = 42u8;
        assert!(check_null_pointer(&x as *const u8));
    }

    #[test]
    fn test_buffer_size_within_bounds() {
        assert!(validate_buffer_size(100, 4096));
    }

    #[test]
    fn test_buffer_size_zero_rejected() {
        assert!(!validate_buffer_size(0, 4096));
    }

    #[test]
    fn test_buffer_size_exceeds_max() {
        assert!(!validate_buffer_size(5000, 4096));
    }
}
