#[cfg(test)]
mod tests {
    use protocol_version_macro::Pvn;

    #[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Pvn)]
    pub enum ProtocolVersion {
        #[default]
        V1_21_4 = 769,
        V1_21_2 = 768,
        V1_21 = 767,
    }

    #[test]
    fn test_get_large_version_number() {
        // Given
        let expected_version_number = ProtocolVersion::V1_21_4;
        let given_number = 800; // Larger than the supported pvn

        // When
        let result = ProtocolVersion::from(given_number);

        // Then
        assert_eq!(result, expected_version_number);
    }

    #[test]
    fn test_get_small_version_number() {
        // Given
        let expected_version_number = ProtocolVersion::V1_21;
        let given_number = 750; // Smaller than the supported pvn

        // When
        let result = ProtocolVersion::from(given_number);

        // Then
        assert_eq!(result, expected_version_number);
    }
}
