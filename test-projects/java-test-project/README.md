# Java Test Project

This test project demonstrates comprehensive Java patterns and features to test the CodePrism MCP server's Java analysis capabilities.

## Project Structure

```
java-test-project/
├── src/main/java/
│   ├── com/codeprism/test/
│   │   ├── Application.java              # Main application entry point
│   │   ├── config/                       # Configuration patterns
│   │   ├── controllers/                  # Spring MVC controllers
│   │   ├── services/                     # Business logic services
│   │   ├── repositories/                 # Data access layer
│   │   ├── models/                       # Domain models and entities
│   │   ├── security/                     # Security implementations
│   │   ├── patterns/                     # Design pattern demonstrations
│   │   └── utils/                        # Utility classes
├── src/test/java/                        # Test classes
├── pom.xml                               # Maven configuration
└── README.md                             # This file
```

## Features Tested

### Object-Oriented Programming Patterns
- Class hierarchies and inheritance
- Polymorphism and method overriding
- Encapsulation and data hiding
- Interface implementations
- Abstract classes and methods

### Design Patterns
- Singleton pattern
- Factory pattern
- Builder pattern
- Observer pattern
- Strategy pattern
- Repository pattern
- MVC pattern

### Spring Framework Features
- Dependency injection (@Autowired, @Component, @Service, @Repository)
- Spring Boot auto-configuration
- REST controllers (@RestController, @RequestMapping)
- Data access with Spring Data JPA
- Security configuration
- AOP (Aspect-Oriented Programming)
- Transaction management

### JPA/Hibernate Features
- Entity mappings (@Entity, @Table, @Column)
- Relationships (@OneToMany, @ManyToOne, @ManyToMany)
- Custom queries (@Query, JPQL)
- Repository interfaces
- Pagination and sorting

### Modern Java Features
- Lambda expressions and functional interfaces
- Stream API usage
- Optional handling
- Method references
- Local variable type inference (var)
- Switch expressions (Java 12+)
- Text blocks (Java 13+)
- Records (Java 14+)

### Security Patterns
- Authentication and authorization
- Input validation
- SQL injection prevention
- XSS protection
- CSRF protection
- Password encoding

### Testing Patterns
- JUnit 5 test cases
- Mockito for mocking
- Integration tests
- Parameterized tests
- Test fixtures and setup

## Usage

This project is designed to be analyzed by the CodePrism MCP server to test all Java-related analysis tools:

- `analyze_complexity` - Test complexity metrics
- `search_symbols` - Find Java classes, methods, variables
- `trace_inheritance` - Analyze class hierarchies
- `find_dependencies` - Trace dependencies between classes
- `analyze_security` - Detect security vulnerabilities
- `analyze_performance` - Identify performance issues
- `detect_patterns` - Find design patterns
- `analyze_javascript_frameworks` - N/A for Java project

## Build Instructions

```bash
# Compile the project
mvn compile

# Run tests
mvn test

# Package as JAR
mvn package

# Run the application
java -jar target/java-test-project-1.0.0.jar
```

## Dependencies

- Spring Boot 3.2.0
- Spring Data JPA
- Spring Security
- H2 Database (for testing)
- JUnit 5
- Mockito
- Hibernate Validator 