package com.codeprism.test.services;

import com.codeprism.test.models.User;
import com.codeprism.test.repositories.UserRepository;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.cache.annotation.CacheEvict;
import org.springframework.cache.annotation.Cacheable;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.scheduling.annotation.Async;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import org.springframework.validation.annotation.Validated;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/**
 * User service layer demonstrating Spring features.
 * 
 * This service showcases:
 * - Dependency injection
 * - Transaction management
 * - Caching annotations
 * - Async processing
 * - Validation
 * - Security integration
 * - Exception handling
 */
@Slf4j
@Service
@Validated
@Transactional(readOnly = true)
public class UserService {

    private final UserRepository userRepository;
    private final PasswordEncoder passwordEncoder;

    /**
     * Constructor injection for dependencies.
     * 
     * @param userRepository the user repository
     * @param passwordEncoder the password encoder
     */
    @Autowired
    public UserService(UserRepository userRepository, PasswordEncoder passwordEncoder) {
        this.userRepository = userRepository;
        this.passwordEncoder = passwordEncoder;
    }

    /**
     * Find all users.
     * 
     * @return list of all users
     */
    public List<User> findAll() {
        log.debug("Finding all users");
        return userRepository.findAll();
    }

    /**
     * Find user by ID with caching.
     * 
     * @param id the user ID
     * @return optional user
     */
    @Cacheable(value = "users", key = "#id")
    public Optional<User> findById(@NotNull Long id) {
        log.debug("Finding user by ID: {}", id);
        return userRepository.findById(id);
    }

    /**
     * Find user by username.
     * 
     * @param username the username
     * @return optional user
     */
    @Cacheable(value = "users", key = "#username")
    public Optional<User> findByUsername(@NotBlank String username) {
        log.debug("Finding user by username: {}", username);
        return userRepository.findByUsername(username);
    }

    /**
     * Find user by email.
     * 
     * @param email the email address
     * @return optional user
     */
    @Cacheable(value = "users", key = "#email")
    public Optional<User> findByEmail(@NotBlank String email) {
        log.debug("Finding user by email: {}", email);
        return userRepository.findByEmail(email);
    }

    /**
     * Find active users only.
     * 
     * @return list of active users
     */
    public List<User> findActiveUsers() {
        log.debug("Finding active users");
        return userRepository.findByActiveTrue();
    }

    /**
     * Find users by role.
     * 
     * @param role the user role
     * @return list of users with the specified role
     */
    public List<User> findByRole(@NotNull User.Role role) {
        log.debug("Finding users with role: {}", role);
        return userRepository.findByRole(role);
    }

    /**
     * Find users with pagination.
     * 
     * @param pageable pagination parameters
     * @return page of users
     */
    public Page<User> findAll(Pageable pageable) {
        log.debug("Finding users with pagination: {}", pageable);
        return userRepository.findAll(pageable);
    }

    /**
     * Create a new user with validation and security.
     * 
     * @param user the user to create
     * @return the created user
     */
    @Transactional
    @CacheEvict(value = "users", allEntries = true)
    public User createUser(@Valid User user) {
        log.info("Creating new user: {}", user.getUsername());

        // Check if username already exists
        if (userRepository.existsByUsername(user.getUsername())) {
            throw new UserServiceException("Username already exists: " + user.getUsername());
        }

        // Check if email already exists
        if (userRepository.existsByEmail(user.getEmail())) {
            throw new UserServiceException("Email already exists: " + user.getEmail());
        }

        // Encode password if provided
        if (user.getPasswordHash() != null) {
            user.setPasswordHash(passwordEncoder.encode(user.getPasswordHash()));
        }

        // Set default values
        if (user.getRole() == null) {
            user.setRole(User.Role.USER);
        }

        try {
            User savedUser = userRepository.save(user);
            log.info("Successfully created user with ID: {}", savedUser.getId());
            return savedUser;
        } catch (Exception e) {
            log.error("Failed to create user: {}", user.getUsername(), e);
            throw new UserServiceException("Failed to create user", e);
        }
    }

    /**
     * Update an existing user.
     * 
     * @param user the user to update
     * @return the updated user
     */
    @Transactional
    @CacheEvict(value = "users", key = "#user.id")
    public User updateUser(@Valid User user) {
        log.info("Updating user: {}", user.getId());

        if (!userRepository.existsById(user.getId())) {
            throw new UserServiceException("User not found with ID: " + user.getId());
        }

        try {
            User updatedUser = userRepository.save(user);
            log.info("Successfully updated user: {}", updatedUser.getId());
            return updatedUser;
        } catch (Exception e) {
            log.error("Failed to update user: {}", user.getId(), e);
            throw new UserServiceException("Failed to update user", e);
        }
    }

    /**
     * Delete a user by ID.
     * 
     * @param id the user ID
     */
    @Transactional
    @CacheEvict(value = "users", key = "#id")
    public void deleteUser(@NotNull Long id) {
        log.info("Deleting user: {}", id);

        if (!userRepository.existsById(id)) {
            throw new UserServiceException("User not found with ID: " + id);
        }

        try {
            userRepository.deleteById(id);
            log.info("Successfully deleted user: {}", id);
        } catch (Exception e) {
            log.error("Failed to delete user: {}", id, e);
            throw new UserServiceException("Failed to delete user", e);
        }
    }

    /**
     * Activate all users in batch.
     */
    @Transactional
    @CacheEvict(value = "users", allEntries = true)
    public void activateAllUsers() {
        log.info("Activating all users");
        
        try {
            int updatedCount = userRepository.activateAllUsers();
            log.info("Activated {} users", updatedCount);
        } catch (Exception e) {
            log.error("Failed to activate all users", e);
            throw new UserServiceException("Failed to activate users", e);
        }
    }

    /**
     * Deactivate inactive users (no login in last 90 days).
     */
    @Transactional
    @CacheEvict(value = "users", allEntries = true)
    public void deactivateInactiveUsers() {
        log.info("Deactivating inactive users");
        
        LocalDateTime cutoffDate = LocalDateTime.now().minusDays(90);
        
        try {
            int deactivatedCount = userRepository.deactivateUsersLastLoginBefore(cutoffDate);
            log.info("Deactivated {} inactive users", deactivatedCount);
        } catch (Exception e) {
            log.error("Failed to deactivate inactive users", e);
            throw new UserServiceException("Failed to deactivate users", e);
        }
    }

    /**
     * Count total users.
     * 
     * @return total user count
     */
    public long countUsers() {
        return userRepository.count();
    }

    /**
     * Count active users.
     * 
     * @return active user count
     */
    public long countActiveUsers() {
        return userRepository.countByActiveTrue();
    }

    /**
     * Async method to find all users.
     * 
     * @return CompletableFuture with list of users
     */
    @Async
    public CompletableFuture<List<User>> findAllAsync() {
        log.debug("Finding all users asynchronously");
        
        try {
            List<User> users = userRepository.findAll();
            return CompletableFuture.completedFuture(users);
        } catch (Exception e) {
            log.error("Async findAll failed", e);
            return CompletableFuture.failedFuture(e);
        }
    }

    /**
     * Async method to count users.
     * 
     * @return CompletableFuture with user count
     */
    @Async
    public CompletableFuture<Long> countUsersAsync() {
        log.debug("Counting users asynchronously");
        
        try {
            long count = userRepository.count();
            return CompletableFuture.completedFuture(count);
        } catch (Exception e) {
            log.error("Async count failed", e);
            return CompletableFuture.failedFuture(e);
        }
    }

    /**
     * Update user's last login timestamp.
     * 
     * @param userId the user ID
     */
    @Transactional
    @CacheEvict(value = "users", key = "#userId")
    public void updateLastLogin(@NotNull Long userId) {
        log.debug("Updating last login for user: {}", userId);
        
        userRepository.findById(userId).ifPresentOrElse(
            user -> {
                user.setLastLogin(LocalDateTime.now());
                user.resetLoginAttempts();
                userRepository.save(user);
            },
            () -> {
                throw new UserServiceException("User not found with ID: " + userId);
            }
        );
    }

    /**
     * Handle failed login attempt.
     * 
     * @param username the username
     */
    @Transactional
    public void handleFailedLogin(@NotBlank String username) {
        log.warn("Handling failed login attempt for user: {}", username);
        
        userRepository.findByUsername(username).ifPresent(user -> {
            user.incrementLoginAttempts();
            userRepository.save(user);
            
            if (user.getAccountLocked()) {
                log.warn("Account locked due to too many failed attempts: {}", username);
            }
        });
    }

    /**
     * Custom exception for user service operations.
     */
    public static class UserServiceException extends RuntimeException {
        public UserServiceException(String message) {
            super(message);
        }

        public UserServiceException(String message, Throwable cause) {
            super(message, cause);
        }
    }
} 