// Calculates the average of a slice of f64 values.
// Returns None if the slice is empty.
pub fn calculate_average(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        None
    } else {
        let sum: f64 = data.iter().sum();
        Some(sum / data.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_average_empty() {
        let data: [f64; 0] = [];
        assert_eq!(calculate_average(&data), None);
    }

    #[test]
    fn test_calculate_average_single() {
        let data = [10.0];
        assert_eq!(calculate_average(&data), Some(10.0));
    }

    #[test]
    fn test_calculate_average_multiple() {
        let data = [10.0, 20.0, 30.0];
        assert_eq!(calculate_average(&data), Some(20.0));
    }

    #[test]
    fn test_calculate_average_negative() {
        let data = [-10.0, 0.0, 10.0];
        // Use assert! with a tolerance for floating-point comparisons, even for zero
        let avg = calculate_average(&data).unwrap();
        assert!((avg - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_average_floating_point() {
        let data = [10.5, 11.5, 12.5];
        // Use assert! with a tolerance for floating-point comparisons
        let avg = calculate_average(&data).unwrap();
        assert!(
            (avg - 11.5).abs() < f64::EPSILON,
            "Expected average to be close to 11.5, but got {}",
            avg
        );
    }

    // Optional: Add a test for a larger dataset if needed
    #[test]
    fn test_calculate_average_larger_set() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let avg = calculate_average(&data).unwrap();
        assert!(
            (avg - 5.5).abs() < f64::EPSILON,
            "Expected average to be close to 5.5, but got {}",
            avg
        );
    }
}
