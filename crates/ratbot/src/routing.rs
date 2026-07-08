use crate::assignment::Assignment;
use serenity::all::UserId;

#[derive(Debug, PartialEq, Eq)]
pub enum RouteTarget {
    Giftee,
    SecretSanta,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RouteError {
    UserNotParticipating,
    AssignmentNotFound,
}

/// Determines the target UserId for an anonymous message.
pub fn route_message(
    sender: UserId,
    target: RouteTarget,
    assignments: &[Assignment],
) -> Result<UserId, RouteError> {
    match target {
        RouteTarget::Giftee => assignments
            .iter()
            .find(|a| a.gifter == sender)
            .map(|a| a.recipient)
            .ok_or(RouteError::UserNotParticipating),
        RouteTarget::SecretSanta => assignments
            .iter()
            .find(|a| a.recipient == sender)
            .map(|a| a.gifter)
            .ok_or(RouteError::UserNotParticipating),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_assignments() -> Vec<Assignment> {
        vec![
            Assignment {
                gifter: UserId::new(1),
                recipient: UserId::new(2),
            },
            Assignment {
                gifter: UserId::new(2),
                recipient: UserId::new(3),
            },
            Assignment {
                gifter: UserId::new(3),
                recipient: UserId::new(1),
            },
        ]
    }

    #[test]
    fn test_route_to_giftee() {
        let assignments = get_test_assignments();
        // User 1's giftee is User 2
        assert_eq!(
            route_message(UserId::new(1), RouteTarget::Giftee, &assignments),
            Ok(UserId::new(2))
        );
        // User 2's giftee is User 3
        assert_eq!(
            route_message(UserId::new(2), RouteTarget::Giftee, &assignments),
            Ok(UserId::new(3))
        );
        // User 3's giftee is User 1
        assert_eq!(
            route_message(UserId::new(3), RouteTarget::Giftee, &assignments),
            Ok(UserId::new(1))
        );
    }

    #[test]
    fn test_route_to_santa() {
        let assignments = get_test_assignments();
        // User 1's santa is User 3 (since 3 gives to 1)
        assert_eq!(
            route_message(UserId::new(1), RouteTarget::SecretSanta, &assignments),
            Ok(UserId::new(3))
        );
        // User 2's santa is User 1
        assert_eq!(
            route_message(UserId::new(2), RouteTarget::SecretSanta, &assignments),
            Ok(UserId::new(1))
        );
        // User 3's santa is User 2
        assert_eq!(
            route_message(UserId::new(3), RouteTarget::SecretSanta, &assignments),
            Ok(UserId::new(2))
        );
    }

    #[test]
    fn test_unrecognized_user() {
        let assignments = get_test_assignments();
        let unknown = UserId::new(999);

        assert_eq!(
            route_message(unknown, RouteTarget::Giftee, &assignments),
            Err(RouteError::UserNotParticipating)
        );

        assert_eq!(
            route_message(unknown, RouteTarget::SecretSanta, &assignments),
            Err(RouteError::UserNotParticipating)
        );
    }
}
