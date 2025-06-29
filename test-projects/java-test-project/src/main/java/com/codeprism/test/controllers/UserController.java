package com.codeprism.test.controllers;

import com.codeprism.test.models.User;
import com.codeprism.test.services.UserService;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.security.access.prepost.PreAuthorize;
import org.springframework.validation.annotation.Validated;
import org.springframework.web.bind.annotation.*;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import java.util.List;
import java.util.Optional;

/**
 * REST Controller for User operations.
 * 
 * This controller demonstrates:
 * - RESTful API design
 * - Spring MVC annotations
 * - Request/Response handling
 * - Validation
 * - Security annotations
 * - Exception handling
 * - Pagination support
 */
@Slf4j
@RestController
@RequestMapping("/api/v1/users")
@Validated
@CrossOrigin(origins = "*", maxAge = 3600)
public class UserController {

    private final UserService userService;

    /**
     * Constructor injection for dependencies.
     * 
     * @param userService the user service
     */
    @Autowired
    public UserController(UserService userService) {
        this.userService = userService;
    }

    /**
     * Get all users with pagination.
     * 
     * @param pageable pagination parameters
     * @return page of users
     */
    @GetMapping
    @PreAuthorize("hasRole('USER')")
    public ResponseEntity<Page<User>> getAllUsers(Pageable pageable) {
        log.info("Fetching users with pagination: {}", pageable);
        
        try {
            Page<User> users = userService.findAll(pageable);
            return ResponseEntity.ok(users);
        } catch (Exception e) {
            log.error("Error fetching users", e);
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
        }
    }

    /**
     * Get user by ID.
     * 
     * @param id the user ID
     * @return user or 404 if not found
     */
    @GetMapping("/{id}")
    @PreAuthorize("hasRole('USER')")
    public ResponseEntity<User> getUserById(@PathVariable @NotNull Long id) {
        log.info("Fetching user by ID: {}", id);
        
        Optional<User> user = userService.findById(id);
        return user.map(ResponseEntity::ok)
                  .orElse(ResponseEntity.notFound().build());
    }

    /**
     * Search users by username.
     * 
     * @param username the username to search for
     * @return list of matching users
     */
    @GetMapping("/search")
    @PreAuthorize("hasRole('USER')")
    public ResponseEntity<List<User>> searchUsers(
            @RequestParam(name = "username", required = false) String username,
            @RequestParam(name = "email", required = false) String email,
            @RequestParam(name = "role", required = false) User.Role role) {
        
        log.info("Searching users with username: {}, email: {}, role: {}", username, email, role);
        
        try {
            List<User> users;
            
            if (username != null) {
                users = userService.findByUsername(username)
                    .map(List::of)
                    .orElse(List.of());
            } else if (email != null) {
                users = userService.findByEmail(email)
                    .map(List::of)
                    .orElse(List.of());
            } else if (role != null) {
                users = userService.findByRole(role);
            } else {
                users = userService.findActiveUsers();
            }
            
            return ResponseEntity.ok(users);
        } catch (Exception e) {
            log.error("Error searching users", e);
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
        }
    }

    /**
     * Create a new user.
     * 
     * @param user the user to create
     * @return created user
     */
    @PostMapping
    @PreAuthorize("hasRole('ADMIN')")
    public ResponseEntity<User> createUser(@Valid @RequestBody User user) {
        log.info("Creating new user: {}", user.getUsername());
        
        try {
            User createdUser = userService.createUser(user);
            return ResponseEntity.status(HttpStatus.CREATED).body(createdUser);
        } catch (UserService.UserServiceException e) {
            log.warn("User creation failed: {}", e.getMessage());
            return ResponseEntity.status(HttpStatus.CONFLICT).build();
        } catch (Exception e) {
            log.error("Error creating user", e);
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
        }
    }

    /**
     * Update an existing user.
     * 
     * @param id the user ID
     * @param user the updated user data
     * @return updated user
     */
    @PutMapping("/{id}")
    @PreAuthorize("hasRole('ADMIN') or #id == authentication.principal.id")
    public ResponseEntity<User> updateUser(@PathVariable Long id, @Valid @RequestBody User user) {
        log.info("Updating user: {}", id);
        
        if (!id.equals(user.getId())) {
            return ResponseEntity.badRequest().build();
        }
        
        try {
            User updatedUser = userService.updateUser(user);
            return ResponseEntity.ok(updatedUser);
        } catch (UserService.UserServiceException e) {
            log.warn("User update failed: {}", e.getMessage());
            return ResponseEntity.notFound().build();
        } catch (Exception e) {
            log.error("Error updating user", e);
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
        }
    }

    /**
     * Delete a user.
     * 
     * @param id the user ID
     * @return no content response
     */
    @DeleteMapping("/{id}")
    @PreAuthorize("hasRole('ADMIN')")
    public ResponseEntity<Void> deleteUser(@PathVariable Long id) {
        log.info("Deleting user: {}", id);
        
        try {
            userService.deleteUser(id);
            return ResponseEntity.noContent().build();
        } catch (UserService.UserServiceException e) {
            log.warn("User deletion failed: {}", e.getMessage());
            return ResponseEntity.notFound().build();
        } catch (Exception e) {
            log.error("Error deleting user", e);
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
        }
    }

    /**
     * Activate a user account.
     * 
     * @param id the user ID
     * @return no content response
     */
    @PatchMapping("/{id}/activate")
    @PreAuthorize("hasRole('ADMIN')")
    public ResponseEntity<Void> activateUser(@PathVariable Long id) {
        log.info("Activating user: {}", id);
        
        try {
            Optional<User> userOpt = userService.findById(id);
            if (userOpt.isEmpty()) {
                return ResponseEntity.notFound().build();
            }
            
            User user = userOpt.get();
            user.setActive(true);
            user.setAccountLocked(false);
            userService.updateUser(user);
            
            return ResponseEntity.noContent().build();
        } catch (Exception e) {
            log.error("Error activating user", e);
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
        }
    }

    /**
     * Deactivate a user account.
     * 
     * @param id the user ID
     * @return no content response
     */
    @PatchMapping("/{id}/deactivate")
    @PreAuthorize("hasRole('ADMIN')")
    public ResponseEntity<Void> deactivateUser(@PathVariable Long id) {
        log.info("Deactivating user: {}", id);
        
        try {
            Optional<User> userOpt = userService.findById(id);
            if (userOpt.isEmpty()) {
                return ResponseEntity.notFound().build();
            }
            
            User user = userOpt.get();
            user.setActive(false);
            userService.updateUser(user);
            
            return ResponseEntity.noContent().build();
        } catch (Exception e) {
            log.error("Error deactivating user", e);
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
        }
    }

    /**
     * Get user statistics.
     * 
     * @return user statistics
     */
    @GetMapping("/stats")
    @PreAuthorize("hasRole('ADMIN')")
    public ResponseEntity<UserStats> getUserStats() {
        log.info("Fetching user statistics");
        
        try {
            long totalUsers = userService.countUsers();
            long activeUsers = userService.countActiveUsers();
            
            return ResponseEntity.ok(new UserStats(totalUsers, activeUsers));
        } catch (Exception e) {
            log.error("Error fetching user statistics", e);
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
        }
    }

    /**
     * Batch activate all users.
     * 
     * @return no content response
     */
    @PostMapping("/batch/activate")
    @PreAuthorize("hasRole('ADMIN')")
    public ResponseEntity<Void> activateAllUsers() {
        log.info("Batch activating all users");
        
        try {
            userService.activateAllUsers();
            return ResponseEntity.noContent().build();
        } catch (Exception e) {
            log.error("Error batch activating users", e);
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
        }
    }

    /**
     * User statistics DTO.
     */
    public static class UserStats {
        public final long totalUsers;
        public final long activeUsers;
        public final long inactiveUsers;
        public final double activePercentage;

        public UserStats(long totalUsers, long activeUsers) {
            this.totalUsers = totalUsers;
            this.activeUsers = activeUsers;
            this.inactiveUsers = totalUsers - activeUsers;
            this.activePercentage = totalUsers > 0 ? (double) activeUsers / totalUsers * 100 : 0;
        }
    }

    /**
     * Exception handler for validation errors.
     * 
     * @param e the validation exception
     * @return error response
     */
    @ExceptionHandler(org.springframework.web.bind.MethodArgumentNotValidException.class)
    public ResponseEntity<String> handleValidationException(
            org.springframework.web.bind.MethodArgumentNotValidException e) {
        
        log.warn("Validation error: {}", e.getMessage());
        return ResponseEntity.badRequest().body("Validation failed: " + e.getMessage());
    }

    /**
     * Exception handler for user service exceptions.
     * 
     * @param e the user service exception
     * @return error response
     */
    @ExceptionHandler(UserService.UserServiceException.class)
    public ResponseEntity<String> handleUserServiceException(UserService.UserServiceException e) {
        log.warn("User service error: {}", e.getMessage());
        return ResponseEntity.status(HttpStatus.CONFLICT).body(e.getMessage());
    }
} 