package com.codeprism.test.models;

import jakarta.persistence.*;
import jakarta.validation.constraints.*;
import lombok.*;
import org.hibernate.annotations.CreationTimestamp;
import org.hibernate.annotations.UpdateTimestamp;
import org.springframework.data.annotation.CreatedDate;
import org.springframework.data.annotation.LastModifiedDate;
import org.springframework.data.jpa.domain.support.AuditingEntityListener;

import java.time.LocalDate;
import java.time.LocalDateTime;
import java.util.HashSet;
import java.util.Set;

/**
 * User entity demonstrating JPA annotations, validation, Lombok, and design patterns.
 * 
 * This class showcases:
 * - JPA entity mapping with relationships
 * - Bean validation annotations
 * - Lombok for boilerplate reduction
 * - Audit trail capabilities
 * - Builder pattern
 * - Enum usage for type safety
 * - Security considerations
 */
@Entity
@Table(name = "users", 
       uniqueConstraints = {
           @UniqueConstraint(columnNames = "username"),
           @UniqueConstraint(columnNames = "email")
       },
       indexes = {
           @Index(name = "idx_user_email", columnList = "email"),
           @Index(name = "idx_user_role", columnList = "role"),
           @Index(name = "idx_user_active", columnList = "active")
       })
@EntityListeners(AuditingEntityListener.class)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@EqualsAndHashCode(exclude = {"addresses", "createdAt", "updatedAt"})
@ToString(exclude = {"addresses"})
public class User {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(nullable = false, length = 50)
    @NotBlank(message = "Username is required")
    @Size(min = 3, max = 50, message = "Username must be between 3 and 50 characters")
    @Pattern(regexp = "^[a-zA-Z0-9_]+$", message = "Username can only contain letters, numbers, and underscores")
    private String username;

    @Column(nullable = false, unique = true, length = 100)
    @NotBlank(message = "Email is required")
    @Email(message = "Email should be valid")
    @Size(max = 100, message = "Email must not exceed 100 characters")
    private String email;

    @Column(name = "first_name", nullable = false, length = 50)
    @NotBlank(message = "First name is required")
    @Size(max = 50, message = "First name must not exceed 50 characters")
    private String firstName;

    @Column(name = "last_name", nullable = false, length = 50)
    @NotBlank(message = "Last name is required")
    @Size(max = 50, message = "Last name must not exceed 50 characters")
    private String lastName;

    @Column(name = "password_hash", nullable = false)
    @NotBlank(message = "Password is required")
    @Size(min = 60, max = 100, message = "Password hash should be between 60-100 characters")
    private String passwordHash;

    @Column(name = "birth_date")
    @Past(message = "Birth date must be in the past")
    private LocalDate birthDate;

    @Column(nullable = false)
    @Enumerated(EnumType.STRING)
    @NotNull(message = "Role is required")
    private Role role;

    @Column(nullable = false)
    @Builder.Default
    private Boolean active = true;

    @Column(name = "login_attempts")
    @Min(value = 0, message = "Login attempts cannot be negative")
    @Builder.Default
    private Integer loginAttempts = 0;

    @Column(name = "last_login")
    private LocalDateTime lastLogin;

    @Column(name = "account_locked")
    @Builder.Default
    private Boolean accountLocked = false;

    @CreationTimestamp
    @CreatedDate
    @Column(name = "created_at", nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @UpdateTimestamp
    @LastModifiedDate
    @Column(name = "updated_at")
    private LocalDateTime updatedAt;

    @OneToMany(mappedBy = "user", cascade = CascadeType.ALL, fetch = FetchType.LAZY, orphanRemoval = true)
    @Builder.Default
    private Set<Address> addresses = new HashSet<>();

    /**
     * User roles with hierarchical permissions.
     */
    public enum Role {
        GUEST(0, "Guest User"),
        USER(1, "Standard User"),
        MANAGER(2, "Manager"),
        ADMIN(3, "Administrator");

        private final int level;
        private final String description;

        Role(int level, String description) {
            this.level = level;
            this.description = description;
        }

        public int getLevel() {
            return level;
        }

        public String getDescription() {
            return description;
        }

        public boolean hasHigherOrEqualAuthorityThan(Role other) {
            return this.level >= other.level;
        }
    }

    /**
     * Business logic: Check if user has permission for a specific role level.
     * 
     * @param requiredRole the minimum required role
     * @return true if user has sufficient permissions
     */
    public boolean hasRole(Role requiredRole) {
        return this.role.hasHigherOrEqualAuthorityThan(requiredRole);
    }

    /**
     * Business logic: Check if account is accessible (active and not locked).
     * 
     * @return true if account can be accessed
     */
    public boolean isAccountAccessible() {
        return Boolean.TRUE.equals(active) && !Boolean.TRUE.equals(accountLocked);
    }

    /**
     * Business logic: Get full name.
     * 
     * @return concatenated first and last name
     */
    public String getFullName() {
        return String.format("%s %s", firstName, lastName);
    }

    /**
     * Business logic: Check if user is adult based on birth date.
     * 
     * @return true if user is 18 or older
     */
    public boolean isAdult() {
        if (birthDate == null) {
            return false;
        }
        return birthDate.isBefore(LocalDate.now().minusYears(18));
    }

    /**
     * Business logic: Check if user is active.
     * 
     * @return true if user is active
     */
    public boolean isActive() {
        return Boolean.TRUE.equals(active);
    }

    /**
     * Security: Increment login attempts counter.
     */
    public void incrementLoginAttempts() {
        this.loginAttempts = (this.loginAttempts == null) ? 1 : this.loginAttempts + 1;
        
        // Auto-lock account after 5 failed attempts
        if (this.loginAttempts >= 5) {
            this.accountLocked = true;
        }
    }

    /**
     * Security: Reset login attempts after successful login.
     */
    public void resetLoginAttempts() {
        this.loginAttempts = 0;
        this.lastLogin = LocalDateTime.now();
        this.accountLocked = false;
    }

    /**
     * Utility: Add address to user's address collection.
     * 
     * @param address the address to add
     */
    public void addAddress(Address address) {
        addresses.add(address);
        address.setUser(this);
    }

    /**
     * Utility: Remove address from user's address collection.
     * 
     * @param address the address to remove
     */
    public void removeAddress(Address address) {
        addresses.remove(address);
        address.setUser(null);
    }

    /**
     * Pre-persist callback for additional validation and setup.
     */
    @PrePersist
    protected void onCreate() {
        if (createdAt == null) {
            createdAt = LocalDateTime.now();
        }
        if (updatedAt == null) {
            updatedAt = LocalDateTime.now();
        }
        if (role == null) {
            role = Role.USER;
        }
        if (active == null) {
            active = true;
        }
        if (loginAttempts == null) {
            loginAttempts = 0;
        }
        if (accountLocked == null) {
            accountLocked = false;
        }
    }

    /**
     * Pre-update callback.
     */
    @PreUpdate
    protected void onUpdate() {
        updatedAt = LocalDateTime.now();
    }
} 