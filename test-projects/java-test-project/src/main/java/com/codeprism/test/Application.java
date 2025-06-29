package com.codeprism.test;

import com.codeprism.test.config.DatabaseConfig;
import com.codeprism.test.models.User;
import com.codeprism.test.services.UserService;
import com.codeprism.test.patterns.SingletonLogger;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.CommandLineRunner;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.cache.annotation.EnableCaching;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;
import org.springframework.data.jpa.repository.config.EnableJpaRepositories;
import org.springframework.scheduling.annotation.EnableAsync;
import org.springframework.transaction.annotation.EnableTransactionManagement;

import java.time.LocalDate;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

/**
 * Main Spring Boot application class for the CodePrism Java test project.
 * 
 * This class demonstrates:
 * - Spring Boot auto-configuration
 * - Dependency injection patterns
 * - Modern Java features (var, Optional, Stream API, Lambda expressions)
 * - Design patterns (Singleton)
 * - Asynchronous programming
 * - Transaction management
 * - JPA repository usage
 * 
 * Security note: This is a test application with hardcoded values.
 * In production, use proper configuration management and secrets handling.
 */
@Slf4j
@SpringBootApplication
@EnableJpaRepositories(basePackages = "com.codeprism.test.repositories")
@EnableTransactionManagement
@EnableAsync
@EnableCaching
@Import(DatabaseConfig.class)
public class Application implements CommandLineRunner {

    private final UserService userService;
    private final SingletonLogger logger;

    /**
     * Constructor injection - preferred over field injection for better testability.
     * 
     * @param userService the user service dependency
     */
    @Autowired
    public Application(UserService userService) {
        this.userService = userService;
        this.logger = SingletonLogger.getInstance();
    }

    /**
     * Main entry point for the application.
     * 
     * @param args command line arguments
     */
    public static void main(String[] args) {
        log.info("Starting CodePrism Java Test Application...");
        SpringApplication.run(Application.class, args);
    }

    /**
     * CommandLineRunner implementation - executes after Spring context initialization.
     * Demonstrates various Java and Spring patterns for analysis.
     * 
     * @param args command line arguments
     */
    @Override
    public void run(String... args) throws Exception {
        log.info("Application started successfully!");
        logger.log("Singleton logger initialized");

        // Demonstrate modern Java features and patterns
        demonstrateModernJavaFeatures();
        
        // Demonstrate Spring features
        demonstrateSpringFeatures();
        
        // Demonstrate asynchronous processing
        demonstrateAsyncProcessing();
        
        // Demonstrate error handling patterns
        demonstrateErrorHandling();
        
        log.info("Application demonstration completed");
    }

    /**
     * Demonstrates modern Java features including:
     * - Local variable type inference (var)
     * - Optional handling
     * - Stream API with lambda expressions
     * - Switch expressions (Java 12+)
     * - Text blocks (Java 13+)
     * - Record classes (Java 14+)
     */
    private void demonstrateModernJavaFeatures() {
        log.info("=== Demonstrating Modern Java Features ===");

        // Local variable type inference (Java 10+)
        var users = userService.findAll();
        var userCount = users.size();
        log.info("Found {} users using var keyword", userCount);

        // Optional handling patterns
        Optional<User> adminUser = userService.findByUsername("admin");
        adminUser.ifPresentOrElse(
            user -> log.info("Admin user found: {}", user.getEmail()),
            () -> log.warn("Admin user not found - creating default admin")
        );

        // Stream API with lambda expressions
        List<String> activeUserEmails = users.stream()
            .filter(User::isActive)
            .map(User::getEmail)
            .filter(email -> email.contains("@company.com"))
            .sorted()
            .collect(Collectors.toList());

        log.info("Active company emails: {}", activeUserEmails);

        // Switch expression (Java 12+) with user role analysis
        for (User user : users) {
            var roleDescription = switch (user.getRole()) {
                case ADMIN -> "Full system access";
                case MANAGER -> "Department management access";
                case USER -> "Standard user access";
                case GUEST -> "Limited read-only access";
                default -> "Unknown role permissions";
            };
            log.debug("User {} has {}", user.getUsername(), roleDescription);
        }

        // Text block example (Java 13+)
        var sqlQuery = """
            SELECT u.username, u.email, u.role
            FROM users u
            WHERE u.active = true
              AND u.created_date > ?
            ORDER BY u.username ASC
            """;
        log.debug("Generated SQL query: {}", sqlQuery);

        // Method reference example
        users.stream()
            .map(User::getUsername)
            .forEach(log::debug);
    }

    /**
     * Demonstrates Spring framework features including:
     * - Service layer interactions
     * - Transaction management
     * - Validation
     * - Caching
     */
    private void demonstrateSpringFeatures() {
        log.info("=== Demonstrating Spring Framework Features ===");

        try {
            // Create sample users with validation
            var newUser = User.builder()
                .username("testuser")
                .email("test@example.com")
                .firstName("Test")
                .lastName("User")
                .birthDate(LocalDate.of(1990, 1, 1))
                .role(User.Role.USER)
                .active(true)
                .build();

            // Transactional service call
            var savedUser = userService.createUser(newUser);
            log.info("Created user with ID: {}", savedUser.getId());

            // Cached service call
            var cachedUser = userService.findById(savedUser.getId());
            cachedUser.ifPresent(user -> log.info("Retrieved cached user: {}", user.getUsername()));

            // Bulk operations with transaction
            userService.activateAllUsers();
            log.info("Activated all users in batch operation");

        } catch (Exception e) {
            log.error("Error in Spring features demonstration", e);
        }
    }

    /**
     * Demonstrates asynchronous processing patterns.
     */
    private void demonstrateAsyncProcessing() {
        log.info("=== Demonstrating Asynchronous Processing ===");

        try {
            // Async service calls
            CompletableFuture<List<User>> futureUsers = userService.findAllAsync();
            CompletableFuture<Long> futureCount = userService.countUsersAsync();

            // Combine async results
            CompletableFuture<String> combinedResult = futureUsers
                .thenCombine(futureCount, (users, count) -> 
                    String.format("Retrieved %d users out of %d total", users.size(), count))
                .exceptionally(throwable -> {
                    log.error("Async operation failed", throwable);
                    return "Failed to retrieve user data";
                });

            // Wait for result (in real app, don't block main thread)
            String result = combinedResult.get();
            log.info("Async result: {}", result);

        } catch (Exception e) {
            log.error("Error in async processing demonstration", e);
        }
    }

    /**
     * Demonstrates error handling patterns including:
     * - Try-with-resources
     * - Custom exceptions
     * - Exception handling in streams
     * - Circuit breaker pattern (simulated)
     */
    private void demonstrateErrorHandling() {
        log.info("=== Demonstrating Error Handling Patterns ===");

        // Try-with-resources pattern (automatic resource management)
        try (var resource = new AutoCloseableResource()) {
            resource.performOperation();
            log.info("Resource operation completed successfully");
        } catch (Exception e) {
            log.error("Resource operation failed", e);
        }

        // Exception handling in streams
        List<String> usernames = List.of("valid_user", "invalid@user", "another_valid");
        var validatedUsers = usernames.stream()
            .filter(this::isValidUsername)
            .collect(Collectors.toList());
        
        log.info("Validated usernames: {}", validatedUsers);

        // Simulated circuit breaker pattern
        try {
            simulateExternalServiceCall();
        } catch (RuntimeException e) {
            log.warn("External service call failed, using fallback: {}", e.getMessage());
            handleServiceFailure();
        }
    }

    /**
     * Validates username format.
     * 
     * @param username the username to validate
     * @return true if valid, false otherwise
     */
    private boolean isValidUsername(String username) {
        try {
            if (username == null || username.trim().isEmpty()) {
                return false;
            }
            
            // Simple validation: alphanumeric and underscore only
            return username.matches("^[a-zA-Z0-9_]+$");
            
        } catch (Exception e) {
            log.debug("Username validation error for '{}': {}", username, e.getMessage());
            return false;
        }
    }

    /**
     * Simulates an external service call that may fail.
     */
    private void simulateExternalServiceCall() {
        // Simulate random failure
        if (Math.random() > 0.7) {
            throw new RuntimeException("External service temporarily unavailable");
        }
        log.info("External service call successful");
    }

    /**
     * Handles service failure with fallback logic.
     */
    private void handleServiceFailure() {
        log.info("Executing fallback logic for service failure");
        // In real application, this might use cached data or alternative service
    }

    /**
     * Auto-closeable resource for demonstrating try-with-resources.
     */
    private static class AutoCloseableResource implements AutoCloseable {
        
        public void performOperation() {
            log.debug("Performing resource operation...");
            // Simulate some work
        }

        @Override
        public void close() throws Exception {
            log.debug("Closing resource...");
            // Cleanup logic here
        }
    }

    /**
     * Bean configuration for demonstration purposes.
     * 
     * @return configured string bean
     */
    @Bean
    public String applicationInfo() {
        return "CodePrism Java Test Application v1.0.0";
    }
} 