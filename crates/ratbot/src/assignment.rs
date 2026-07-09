use rand::seq::SliceRandom;
use serenity::all::UserId;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub struct Assignment {
    pub gifter: UserId,
    pub recipient: UserId,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentError {
    NotEnoughParticipants,
    DuplicateParticipants,
}

/// Generates SecretRat assignments for a list of participants.
/// Ensures no one is assigned to themselves, and everyone gives exactly one gift
/// and receives exactly one gift.
pub fn generate_assignments(participants: &[UserId]) -> Result<Vec<Assignment>, AssignmentError> {
    if participants.len() < 3 {
        return Err(AssignmentError::NotEnoughParticipants);
    }

    let unique_participants: HashSet<_> = participants.iter().collect();
    if unique_participants.len() != participants.len() {
        return Err(AssignmentError::DuplicateParticipants);
    }

    let mut shuffled = participants.to_vec();
    shuffled.shuffle(&mut rand::thread_rng());

    let mut assignments = Vec::with_capacity(shuffled.len());
    for i in 0..shuffled.len() {
        let gifter = shuffled[i];
        let recipient = shuffled[(i + 1) % shuffled.len()];
        assignments.push(Assignment { gifter, recipient });
    }

    Ok(assignments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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

    #[test]
    fn test_duplicate_participants() {
        let duplicates = vec![
            UserId::new(1),
            UserId::new(2),
            UserId::new(3),
            UserId::new(1),
        ];

        assert_eq!(
            generate_assignments(&duplicates),
            Err(AssignmentError::DuplicateParticipants)
        );
    }
}
