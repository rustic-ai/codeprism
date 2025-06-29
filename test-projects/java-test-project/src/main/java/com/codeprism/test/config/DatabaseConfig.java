package com.codeprism.test.config;

import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Profile;
import org.springframework.data.jpa.repository.config.EnableJpaAuditing;
import org.springframework.security.crypto.bcrypt.BCryptPasswordEncoder;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.transaction.annotation.EnableTransactionManagement;

import javax.sql.DataSource;
import org.springframework.boot.jdbc.DataSourceBuilder;

/**
 * Database configuration class demonstrating Spring configuration patterns.
 * 
 * This configuration showcases:
 * - Bean definitions
 * - Property injection
 * - Profile-specific configurations
 * - JPA auditing setup
 * - Security beans
 * - Transaction management
 */
@Slf4j
@Configuration
@EnableJpaAuditing
@EnableTransactionManagement
public class DatabaseConfig {

    @Value("${app.database.url:jdbc:h2:mem:testdb}")
    private String databaseUrl;

    @Value("${app.database.username:sa}")
    private String databaseUsername;

    @Value("${app.database.password:}")
    private String databasePassword;

    @Value("${app.database.driver:org.h2.Driver}")
    private String databaseDriver;

    /**
     * Production DataSource configuration.
     * 
     * @return configured DataSource for production
     */
    @Bean
    @Profile("prod")
    public DataSource productionDataSource() {
        log.info("Configuring production DataSource");
        
        return DataSourceBuilder.create()
            .url(databaseUrl)
            .username(databaseUsername)
            .password(databasePassword)
            .driverClassName(databaseDriver)
            .build();
    }

    /**
     * Development DataSource configuration.
     * 
     * @return configured DataSource for development
     */
    @Bean
    @Profile({"dev", "default"})
    public DataSource developmentDataSource() {
        log.info("Configuring development DataSource with H2 in-memory database");
        
        return DataSourceBuilder.create()
            .url("jdbc:h2:mem:devdb;DB_CLOSE_DELAY=-1;DB_CLOSE_ON_EXIT=FALSE")
            .username("sa")
            .password("")
            .driverClassName("org.h2.Driver")
            .build();
    }

    /**
     * Test DataSource configuration.
     * 
     * @return configured DataSource for testing
     */
    @Bean
    @Profile("test")
    public DataSource testDataSource() {
        log.info("Configuring test DataSource with in-memory H2");
        
        return DataSourceBuilder.create()
            .url("jdbc:h2:mem:testdb;DB_CLOSE_DELAY=-1;DB_CLOSE_ON_EXIT=FALSE")
            .username("sa")
            .password("")
            .driverClassName("org.h2.Driver")
            .build();
    }

    /**
     * Password encoder bean for security.
     * 
     * @return BCrypt password encoder
     */
    @Bean
    public PasswordEncoder passwordEncoder() {
        log.debug("Creating BCrypt password encoder");
        return new BCryptPasswordEncoder(12);
    }

    /**
     * Database connection properties bean.
     * 
     * @return database properties
     */
    @Bean
    public DatabaseProperties databaseProperties() {
        DatabaseProperties properties = new DatabaseProperties();
        properties.setUrl(databaseUrl);
        properties.setUsername(databaseUsername);
        properties.setDriver(databaseDriver);
        // Don't log or expose password
        log.info("Database properties configured for URL: {}", databaseUrl);
        return properties;
    }

    /**
     * Database properties holder class.
     */
    public static class DatabaseProperties {
        private String url;
        private String username;
        private String driver;

        // Getters and setters
        public String getUrl() {
            return url;
        }

        public void setUrl(String url) {
            this.url = url;
        }

        public String getUsername() {
            return username;
        }

        public void setUsername(String username) {
            this.username = username;
        }

        public String getDriver() {
            return driver;
        }

        public void setDriver(String driver) {
            this.driver = driver;
        }

        @Override
        public String toString() {
            return String.format("DatabaseProperties{url='%s', username='%s', driver='%s'}", 
                               url, username, driver);
        }
    }
} 