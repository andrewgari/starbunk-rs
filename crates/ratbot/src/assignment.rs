use serenity::all::UserId;

#[derive(Debug, PartialEq, Eq)]
pub struct Assignment {
    pub gifter: UserId,
    pub recipient: UserId,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentError {
    NotEnoughParticipants,
}

/// Generates Secret Santa assignments for a list of participants.
/// Ensures no one is assigned to themselves, and everyone gives exactly one gift
/// and receives exactly one gift.
pub fn generate_assignments(_participants: &[UserId]) -> Result<Vec<Assignment>, AssignmentError> {
    unimplemented!("Assignment logic not yet implemented for TDD PR 1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "TDD PR 1: logic not implemented yet"]
    fn test_valid_assignment_even() {
        let participants = vec![
            UserId::new(1),
            UserId::new(2),
            UserId::new(3),
            UserId::new(4),
        ];

        let result = generate_assignments(&participants);
        assert!(result.is_ok());

        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 4);

        // Ensure everyone gives one and receives one
        let mut gifters = assignments.iter().map(|a| a.gifter).collect::<Vec<_>>();
        let mut recipients = assignments.iter().map(|a| a.recipient).collect::<Vec<_>>();
        gifters.sort();
        recipients.sort();

        let mut expected = participants.clone();
        expected.sort();

        assert_eq!(gifters, expected);
        assert_eq!(recipients, expected);
    }

    #[test]
    #[ignore = "TDD PR 1: logic not implemented yet"]
    fn test_valid_assignment_odd() {
        let participants = vec![
            UserId::new(1),
            UserId::new(2),
            UserId::new(3),
            UserId::new(4),
            UserId::new(5),
        ];

        let result = generate_assignments(&participants);
        assert!(result.is_ok());

        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 5);

        // Ensure everyone gives one and receives one
        let mut gifters = assignments.iter().map(|a| a.gifter).collect::<Vec<_>>();
        let mut recipients = assignments.iter().map(|a| a.recipient).collect::<Vec<_>>();
        gifters.sort();
        recipients.sort();

        let mut expected = participants.clone();
        expected.sort();

        assert_eq!(gifters, expected);
        assert_eq!(recipients, expected);
    }

    #[test]
    #[ignore = "TDD PR 1: logic not implemented yet"]
    fn test_no_self_assignment() {
        let participants = vec![
            UserId::new(1),
            UserId::new(2),
            UserId::new(3),
            UserId::new(4),
            UserId::new(5),
            UserId::new(6),
        ];

        let result = generate_assignments(&participants);
        assert!(result.is_ok());

        for assignment in result.unwrap() {
            assert_ne!(
                assignment.gifter, assignment.recipient,
                "User was assigned to themselves"
            );
        }
    }

    #[test]
    #[ignore = "TDD PR 1: logic not implemented yet"]
    fn test_not_enough_participants() {
        let one = vec![UserId::new(1)];
        assert_eq!(
            generate_assignments(&one),
            Err(AssignmentError::NotEnoughParticipants)
        );

        let two = vec![UserId::new(1), UserId::new(2)];
        assert_eq!(
            generate_assignments(&two),
            Err(AssignmentError::NotEnoughParticipants)
        );
    }
}
