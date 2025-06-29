package com.codeprism.test.repositories;

import com.codeprism.test.models.User;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.Pageable;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Modifying;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;
import org.springframework.stereotype.Repository;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;

/**
 * Repository interface for User entity operations.
 * 
 * This repository demonstrates:
 * - Spring Data JPA derived queries
 * - Custom JPQL queries
 * - Native SQL queries
 * - Batch operations
 * - Pagination support
 */
@Repository
public interface UserRepository extends JpaRepository<User, Long> {

    /**
     * Find user by username (case-insensitive).
     * 
     * @param username the username
     * @return optional user
     */
    Optional<User> findByUsernameIgnoreCase(String username);

    /**
     * Find user by username (case-sensitive).
     * 
     * @param username the username
     * @return optional user
     */
    Optional<User> findByUsername(String username);

    /**
     * Find user by email (case-insensitive).
     * 
     * @param email the email address
     * @return optional user
     */
    Optional<User> findByEmailIgnoreCase(String email);

    /**
     * Find user by email (case-sensitive).
     * 
     * @param email the email address
     * @return optional user
     */
    Optional<User> findByEmail(String email);

    /**
     * Find all active users.
     * 
     * @return list of active users
     */
    List<User> findByActiveTrue();

    /**
     * Find all inactive users.
     * 
     * @return list of inactive users
     */
    List<User> findByActiveFalse();

    /**
     * Find users by role.
     * 
     * @param role the user role
     * @return list of users with the specified role
     */
    List<User> findByRole(User.Role role);

    /**
     * Find users by role with pagination.
     * 
     * @param role the user role
     * @param pageable pagination parameters
     * @return page of users with the specified role
     */
    Page<User> findByRole(User.Role role, Pageable pageable);

    /**
     * Find active users by role.
     * 
     * @param role the user role
     * @return list of active users with the specified role
     */
    List<User> findByRoleAndActiveTrue(User.Role role);

    /**
     * Find users whose username contains the specified string.
     * 
     * @param username partial username
     * @return list of matching users
     */
    List<User> findByUsernameContainingIgnoreCase(String username);

    /**
     * Find users whose email contains the specified domain.
     * 
     * @param domain email domain
     * @return list of matching users
     */
    List<User> findByEmailContainingIgnoreCase(String domain);

    /**
     * Check if username exists.
     * 
     * @param username the username
     * @return true if exists
     */
    boolean existsByUsername(String username);

    /**
     * Check if email exists.
     * 
     * @param email the email address
     * @return true if exists
     */
    boolean existsByEmail(String email);

    /**
     * Count active users.
     * 
     * @return number of active users
     */
    long countByActiveTrue();

    /**
     * Count users by role.
     * 
     * @param role the user role
     * @return number of users with the specified role
     */
    long countByRole(User.Role role);

    /**
     * Find users with locked accounts.
     * 
     * @return list of users with locked accounts
     */
    List<User> findByAccountLockedTrue();

    /**
     * Find users who have never logged in.
     * 
     * @return list of users with null last login
     */
    List<User> findByLastLoginIsNull();

    /**
     * Find users who logged in after a specific date.
     * 
     * @param date the date threshold
     * @return list of users who logged in after the date
     */
    List<User> findByLastLoginAfter(LocalDateTime date);

    /**
     * Find users who logged in before a specific date.
     * 
     * @param date the date threshold
     * @return list of users who logged in before the date
     */
    List<User> findByLastLoginBefore(LocalDateTime date);

    /**
     * Custom JPQL query to find users by full name.
     * 
     * @param firstName the first name
     * @param lastName the last name
     * @return list of matching users
     */
    @Query("SELECT u FROM User u WHERE u.firstName = :firstName AND u.lastName = :lastName")
    List<User> findByFullName(@Param("firstName") String firstName, @Param("lastName") String lastName);

    /**
     * Custom JPQL query to find users with high login attempts.
     * 
     * @param threshold the login attempts threshold
     * @return list of users with high login attempts
     */
    @Query("SELECT u FROM User u WHERE u.loginAttempts >= :threshold")
    List<User> findUsersWithHighLoginAttempts(@Param("threshold") Integer threshold);

    /**
     * Custom JPQL query to find recently created users.
     * 
     * @param days number of days back to search
     * @return list of recently created users
     */
    @Query("SELECT u FROM User u WHERE u.createdAt >= :cutoffDate")
    List<User> findRecentlyCreatedUsers(@Param("cutoffDate") LocalDateTime cutoffDate);

    /**
     * Native SQL query to find users by domain.
     * 
     * @param domain the email domain
     * @return list of users from the specified domain
     */
    @Query(value = "SELECT * FROM users WHERE email LIKE CONCAT('%@', :domain)", nativeQuery = true)
    List<User> findUsersByEmailDomain(@Param("domain") String domain);

    /**
     * Custom query to get user statistics.
     * 
     * @return user statistics as object array
     */
    @Query("SELECT u.role, COUNT(u), AVG(CAST(u.loginAttempts AS float)) FROM User u GROUP BY u.role")
    List<Object[]> getUserStatistics();

    /**
     * Batch operation to activate all users.
     * 
     * @return number of updated records
     */
    @Modifying
    @Query("UPDATE User u SET u.active = true")
    int activateAllUsers();

    /**
     * Batch operation to deactivate users who haven't logged in recently.
     * 
     * @param cutoffDate the cutoff date for last login
     * @return number of updated records
     */
    @Modifying
    @Query("UPDATE User u SET u.active = false WHERE u.lastLogin < :cutoffDate OR u.lastLogin IS NULL")
    int deactivateUsersLastLoginBefore(@Param("cutoffDate") LocalDateTime cutoffDate);

    /**
     * Batch operation to unlock all locked accounts.
     * 
     * @return number of updated records
     */
    @Modifying
    @Query("UPDATE User u SET u.accountLocked = false, u.loginAttempts = 0")
    int unlockAllAccounts();

    /**
     * Batch operation to reset login attempts for all users.
     * 
     * @return number of updated records
     */
    @Modifying
    @Query("UPDATE User u SET u.loginAttempts = 0")
    int resetAllLoginAttempts();

    /**
     * Delete inactive users created before a specific date.
     * 
     * @param cutoffDate the cutoff date
     * @return number of deleted records
     */
    @Modifying
    @Query("DELETE FROM User u WHERE u.active = false AND u.createdAt < :cutoffDate")
    int deleteInactiveUsersCreatedBefore(@Param("cutoffDate") LocalDateTime cutoffDate);
} 