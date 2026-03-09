use std::fmt;
use std::num::ParseIntError;

#[derive(Debug, PartialEq, Eq)]
pub struct PaginationButton {
    pub command: String,
    pub direction: String,
    pub target_id: u64,
    pub current_page: usize,
}

impl PaginationButton {
    /// Resolves the new page number based on the button direction and pagination state.
    ///
    /// # Arguments
    /// * `total_entries` - Total number of entries to paginate
    /// * `entries_per_page` - Number of entries displayed per page
    ///
    /// # Returns
    /// The new page number (0-indexed) to navigate to
    ///
    /// # Examples
    /// ```
    /// # use pfp_checker::util::pagination::PaginationButton;
    /// let button = PaginationButton {
    ///     command: "pfphistory".to_string(),
    ///     direction: "next".to_string(),
    ///     target_id: 123456,
    ///     current_page: 2,
    /// };
    /// let new_page = button.resolve_new_page(50, 10);
    /// assert_eq!(new_page, 3);
    /// ```
    pub fn resolve_new_page(&self, total_entries: usize, entries_per_page: usize) -> usize {
        let total_pages = if total_entries == 0 {
            0
        } else {
            ((total_entries as f32) / (entries_per_page as f32)).ceil() as usize
        };

        match self.direction.as_str() {
            "first" => 0,
            "back" => self.current_page.saturating_sub(1),
            "next" => self.current_page + 1,
            "last" => total_pages.saturating_sub(1),
            _ => self.current_page,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PaginationParseError {
    InvalidFormat(String),
    InvalidUserId(ParseIntError),
    InvalidPage(ParseIntError),
    UnsupportedCommand(String),
}

impl fmt::Display for PaginationParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaginationParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            PaginationParseError::InvalidUserId(err) => write!(f, "Invalid user ID: {}", err),
            PaginationParseError::InvalidPage(err) => write!(f, "Invalid page number: {}", err),
            PaginationParseError::UnsupportedCommand(cmd) => {
                write!(f, "Unsupported command: {}", cmd)
            }
        }
    }
}

impl std::error::Error for PaginationParseError {}

/// Parses a pagination button custom_id into structured components.
///
/// Supports two formats:
/// - 3-part: `{command}_first_{target_id}` or `{command}_last_{target_id}`
/// - 4-part: `{command}_{back|next}_{page}_{target_id}`
///
/// The `target_id` can be either a user ID (for pfphistory/usernamehistory)
/// or a guild ID (for serverpfphistory).
///
/// # Arguments
/// * `custom_id` - The button's custom_id string
///
/// # Returns
/// * `Ok(PaginationButton)` - Successfully parsed button data
/// * `Err(PaginationParseError)` - Parsing failed
///
/// # Examples
/// ```no_run
/// use pfp_checker::util::pagination::parse_pagination_button;
///
/// let result = parse_pagination_button("pfphistory_first_123456");
/// assert!(result.is_ok());
/// ```
pub fn parse_pagination_button(custom_id: &str) -> Result<PaginationButton, PaginationParseError> {
    let parts: Vec<&str> = custom_id.split('_').collect();

    if parts.len() < 3 {
        return Err(PaginationParseError::InvalidFormat(format!(
            "Expected at least 3 parts, got {}",
            parts.len()
        )));
    }

    let command = parts[0];
    let direction = parts[1];

    // Validate command is supported
    match command {
        "pfphistory" | "usernamehistory" | "serverpfphistory" => {}
        _ => {
            return Err(PaginationParseError::UnsupportedCommand(
                command.to_string(),
            ))
        }
    }

    // Parse based on direction (determines format)
    let (target_id, current_page) = if direction == "first" || direction == "last" {
        // 3-part format: {command}_{first|last}_{target_id}
        if parts.len() != 3 {
            return Err(PaginationParseError::InvalidFormat(format!(
                "Expected 3 parts for first/last button, got {}",
                parts.len()
            )));
        }
        let target_id = parts[2]
            .parse::<u64>()
            .map_err(PaginationParseError::InvalidUserId)?;
        (target_id, 0)
    } else {
        // 4-part format: {command}_{back|next}_{page}_{target_id}
        if parts.len() != 4 {
            return Err(PaginationParseError::InvalidFormat(format!(
                "Expected 4 parts for back/next button, got {}",
                parts.len()
            )));
        }
        let page = parts[2]
            .parse::<usize>()
            .map_err(PaginationParseError::InvalidPage)?;
        let target_id = parts[3]
            .parse::<u64>()
            .map_err(PaginationParseError::InvalidUserId)?;
        (target_id, page)
    };

    Ok(PaginationButton {
        command: command.to_string(),
        direction: direction.to_string(),
        target_id,
        current_page,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pfphistory_first_button() {
        let result = parse_pagination_button("pfphistory_first_123456789");
        assert!(result.is_ok());
        let button = result.unwrap();
        assert_eq!(button.command, "pfphistory");
        assert_eq!(button.direction, "first");
        assert_eq!(button.target_id, 123456789);
        assert_eq!(button.current_page, 0);
    }

    #[test]
    fn test_parse_pfphistory_last_button() {
        let result = parse_pagination_button("pfphistory_last_987654321");
        assert!(result.is_ok());
        let button = result.unwrap();
        assert_eq!(button.command, "pfphistory");
        assert_eq!(button.direction, "last");
        assert_eq!(button.target_id, 987654321);
        assert_eq!(button.current_page, 0);
    }

    #[test]
    fn test_parse_pfphistory_back_button() {
        let result = parse_pagination_button("pfphistory_back_5_123456789");
        assert!(result.is_ok());
        let button = result.unwrap();
        assert_eq!(button.command, "pfphistory");
        assert_eq!(button.direction, "back");
        assert_eq!(button.target_id, 123456789);
        assert_eq!(button.current_page, 5);
    }

    #[test]
    fn test_parse_pfphistory_next_button() {
        let result = parse_pagination_button("pfphistory_next_2_555666777");
        assert!(result.is_ok());
        let button = result.unwrap();
        assert_eq!(button.command, "pfphistory");
        assert_eq!(button.direction, "next");
        assert_eq!(button.target_id, 555666777);
        assert_eq!(button.current_page, 2);
    }

    #[test]
    fn test_parse_usernamehistory_first_button() {
        let result = parse_pagination_button("usernamehistory_first_111222333");
        assert!(result.is_ok());
        let button = result.unwrap();
        assert_eq!(button.command, "usernamehistory");
        assert_eq!(button.direction, "first");
        assert_eq!(button.target_id, 111222333);
        assert_eq!(button.current_page, 0);
    }

    #[test]
    fn test_parse_usernamehistory_next_button() {
        let result = parse_pagination_button("usernamehistory_next_3_444555666");
        assert!(result.is_ok());
        let button = result.unwrap();
        assert_eq!(button.command, "usernamehistory");
        assert_eq!(button.direction, "next");
        assert_eq!(button.target_id, 444555666);
        assert_eq!(button.current_page, 3);
    }

    #[test]
    fn test_parse_serverpfphistory_last_button() {
        let result = parse_pagination_button("serverpfphistory_last_777888999");
        assert!(result.is_ok());
        let button = result.unwrap();
        assert_eq!(button.command, "serverpfphistory");
        assert_eq!(button.direction, "last");
        assert_eq!(button.target_id, 777888999);
        assert_eq!(button.current_page, 0);
    }

    #[test]
    fn test_parse_invalid_too_few_parts() {
        let result = parse_pagination_button("pfphistory_first");
        assert!(result.is_err());
        match result.unwrap_err() {
            PaginationParseError::InvalidFormat(_) => {}
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_parse_invalid_user_id() {
        let result = parse_pagination_button("pfphistory_first_notanumber");
        assert!(result.is_err());
        match result.unwrap_err() {
            PaginationParseError::InvalidUserId(_) => {}
            _ => panic!("Expected InvalidUserId error"),
        }
    }

    #[test]
    fn test_parse_invalid_page_number() {
        let result = parse_pagination_button("pfphistory_back_notanumber_123456");
        assert!(result.is_err());
        match result.unwrap_err() {
            PaginationParseError::InvalidPage(_) => {}
            _ => panic!("Expected InvalidPage error"),
        }
    }

    #[test]
    fn test_parse_unsupported_command() {
        let result = parse_pagination_button("unknowncommand_first_123456");
        assert!(result.is_err());
        match result.unwrap_err() {
            PaginationParseError::UnsupportedCommand(cmd) => {
                assert_eq!(cmd, "unknowncommand");
            }
            _ => panic!("Expected UnsupportedCommand error"),
        }
    }

    #[test]
    fn test_parse_first_button_with_too_many_parts() {
        let result = parse_pagination_button("pfphistory_first_123456_extra");
        assert!(result.is_err());
        match result.unwrap_err() {
            PaginationParseError::InvalidFormat(_) => {}
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_parse_back_button_with_too_few_parts() {
        let result = parse_pagination_button("pfphistory_back_123456");
        assert!(result.is_err());
        match result.unwrap_err() {
            PaginationParseError::InvalidFormat(_) => {}
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_parse_page_zero() {
        let result = parse_pagination_button("pfphistory_next_0_123456");
        assert!(result.is_ok());
        let button = result.unwrap();
        assert_eq!(button.current_page, 0);
    }

    #[test]
    fn test_parse_large_user_id() {
        let result = parse_pagination_button("pfphistory_first_18446744073709551615");
        assert!(result.is_ok());
        let button = result.unwrap();
        assert_eq!(button.target_id, u64::MAX);
    }

    #[test]
    fn test_resolve_new_page_first() {
        let button = PaginationButton {
            command: "pfphistory".to_string(),
            direction: "first".to_string(),
            target_id: 123456,
            current_page: 5,
        };
        assert_eq!(button.resolve_new_page(100, 10), 0);
    }

    #[test]
    fn test_resolve_new_page_last() {
        let button = PaginationButton {
            command: "pfphistory".to_string(),
            direction: "last".to_string(),
            target_id: 123456,
            current_page: 0,
        };
        // 25 entries at 10 per page = 3 pages (0, 1, 2)
        assert_eq!(button.resolve_new_page(25, 10), 2);
    }

    #[test]
    fn test_resolve_new_page_next() {
        let button = PaginationButton {
            command: "pfphistory".to_string(),
            direction: "next".to_string(),
            target_id: 123456,
            current_page: 2,
        };
        assert_eq!(button.resolve_new_page(100, 10), 3);
    }

    #[test]
    fn test_resolve_new_page_back() {
        let button = PaginationButton {
            command: "pfphistory".to_string(),
            direction: "back".to_string(),
            target_id: 123456,
            current_page: 3,
        };
        assert_eq!(button.resolve_new_page(100, 10), 2);
    }

    #[test]
    fn test_resolve_new_page_back_at_zero() {
        let button = PaginationButton {
            command: "pfphistory".to_string(),
            direction: "back".to_string(),
            target_id: 123456,
            current_page: 0,
        };
        // Should saturate at 0, not underflow
        assert_eq!(button.resolve_new_page(100, 10), 0);
    }

    #[test]
    fn test_resolve_new_page_empty_entries() {
        let button = PaginationButton {
            command: "pfphistory".to_string(),
            direction: "last".to_string(),
            target_id: 123456,
            current_page: 0,
        };
        // Empty entries should give page 0
        assert_eq!(button.resolve_new_page(0, 10), 0);
    }

    #[test]
    fn test_resolve_new_page_unknown_direction() {
        let button = PaginationButton {
            command: "pfphistory".to_string(),
            direction: "unknown".to_string(),
            target_id: 123456,
            current_page: 5,
        };
        // Unknown direction should return current page
        assert_eq!(button.resolve_new_page(100, 10), 5);
    }
}
