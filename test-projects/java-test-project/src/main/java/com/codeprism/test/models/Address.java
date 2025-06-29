package com.codeprism.test.models;

import jakarta.persistence.*;
import jakarta.validation.constraints.*;
import lombok.*;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;

import java.time.LocalDateTime;

/**
 * Address entity demonstrating JPA relationships and validation.
 * 
 * This class showcases:
 * - Many-to-One relationship with User
 * - Enum for type safety
 * - Validation annotations
 * - Audit fields
 */
@Entity
@Table(name = "addresses")
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@EqualsAndHashCode(exclude = {"user"})
@ToString(exclude = {"user"})
public class Address {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(nullable = false)
    @Enumerated(EnumType.STRING)
    @NotNull(message = "Address type is required")
    private AddressType type;

    @Column(name = "street_line1", nullable = false, length = 100)
    @NotBlank(message = "Street address is required")
    @Size(max = 100, message = "Street address must not exceed 100 characters")
    private String streetLine1;

    @Column(name = "street_line2", length = 100)
    @Size(max = 100, message = "Street address line 2 must not exceed 100 characters")
    private String streetLine2;

    @Column(nullable = false, length = 50)
    @NotBlank(message = "City is required")
    @Size(max = 50, message = "City must not exceed 50 characters")
    private String city;

    @Column(name = "state_province", length = 50)
    @Size(max = 50, message = "State/Province must not exceed 50 characters")
    private String stateProvince;

    @Column(name = "postal_code", nullable = false, length = 20)
    @NotBlank(message = "Postal code is required")
    @Size(max = 20, message = "Postal code must not exceed 20 characters")
    private String postalCode;

    @Column(nullable = false, length = 50)
    @NotBlank(message = "Country is required")
    @Size(max = 50, message = "Country must not exceed 50 characters")
    private String country;

    @Column(name = "is_primary")
    @Builder.Default
    private Boolean isPrimary = false;

    @CreationTimestamp
    @Column(name = "created_at", nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @UpdateTimestamp
    @Column(name = "updated_at")
    private LocalDateTime updatedAt;

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "user_id", nullable = false)
    private User user;

    /**
     * Address types for categorization.
     */
    public enum AddressType {
        HOME("Home Address"),
        WORK("Work Address"),
        BILLING("Billing Address"),
        SHIPPING("Shipping Address"),
        OTHER("Other Address");

        private final String description;

        AddressType(String description) {
            this.description = description;
        }

        public String getDescription() {
            return description;
        }
    }

    /**
     * Get formatted address string.
     * 
     * @return formatted address
     */
    public String getFormattedAddress() {
        StringBuilder formatted = new StringBuilder();
        formatted.append(streetLine1);
        
        if (streetLine2 != null && !streetLine2.trim().isEmpty()) {
            formatted.append(", ").append(streetLine2);
        }
        
        formatted.append(", ").append(city);
        
        if (stateProvince != null && !stateProvince.trim().isEmpty()) {
            formatted.append(", ").append(stateProvince);
        }
        
        formatted.append(" ").append(postalCode);
        formatted.append(", ").append(country);
        
        return formatted.toString();
    }
} 