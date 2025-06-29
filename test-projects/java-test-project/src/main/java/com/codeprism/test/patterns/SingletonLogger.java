package com.codeprism.test.patterns;

import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.locks.ReentrantReadWriteLock;

/**
 * Singleton Logger demonstrating the Singleton design pattern.
 * 
 * This class showcases:
 * - Thread-safe Singleton implementation (Bill Pugh solution)
 * - Concurrent logging with thread safety
 * - Internal log buffer with size management
 * - Lazy initialization
 * - Enum-based singleton alternative
 */
public class SingletonLogger {

    private static final int MAX_LOG_ENTRIES = 1000;
    private static final DateTimeFormatter TIMESTAMP_FORMAT = 
        DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss.SSS");

    // Thread-safe log storage
    private final ConcurrentLinkedQueue<LogEntry> logBuffer;
    private final ReentrantReadWriteLock lock;

    /**
     * Private constructor to prevent instantiation.
     */
    private SingletonLogger() {
        this.logBuffer = new ConcurrentLinkedQueue<>();
        this.lock = new ReentrantReadWriteLock();
    }

    /**
     * Bill Pugh Singleton Design - Thread-safe lazy initialization
     * without synchronization overhead.
     */
    private static class SingletonHelper {
        private static final SingletonLogger INSTANCE = new SingletonLogger();
    }

    /**
     * Get the singleton instance.
     * 
     * @return the singleton logger instance
     */
    public static SingletonLogger getInstance() {
        return SingletonHelper.INSTANCE;
    }

    /**
     * Log a message with timestamp.
     * 
     * @param message the message to log
     */
    public void log(String message) {
        log(LogLevel.INFO, message);
    }

    /**
     * Log a message with specific level.
     * 
     * @param level the log level
     * @param message the message to log
     */
    public void log(LogLevel level, String message) {
        LogEntry entry = new LogEntry(level, message, LocalDateTime.now());
        
        lock.writeLock().lock();
        try {
            logBuffer.offer(entry);
            
            // Maintain buffer size limit
            while (logBuffer.size() > MAX_LOG_ENTRIES) {
                logBuffer.poll();
            }
        } finally {
            lock.writeLock().unlock();
        }

        // Also output to console for visibility
        System.out.println(formatLogEntry(entry));
    }

    /**
     * Get recent log entries.
     * 
     * @param count number of recent entries to retrieve
     * @return array of recent log entries
     */
    public LogEntry[] getRecentLogs(int count) {
        lock.readLock().lock();
        try {
            return logBuffer.stream()
                .skip(Math.max(0, logBuffer.size() - count))
                .toArray(LogEntry[]::new);
        } finally {
            lock.readLock().unlock();
        }
    }

    /**
     * Get all log entries.
     * 
     * @return array of all log entries
     */
    public LogEntry[] getAllLogs() {
        lock.readLock().lock();
        try {
            return logBuffer.toArray(new LogEntry[0]);
        } finally {
            lock.readLock().unlock();
        }
    }

    /**
     * Clear all log entries.
     */
    public void clearLogs() {
        lock.writeLock().lock();
        try {
            logBuffer.clear();
        } finally {
            lock.writeLock().unlock();
        }
    }

    /**
     * Get current log count.
     * 
     * @return number of log entries
     */
    public int getLogCount() {
        return logBuffer.size();
    }

    /**
     * Format a log entry for display.
     * 
     * @param entry the log entry
     * @return formatted string
     */
    private String formatLogEntry(LogEntry entry) {
        return String.format("[%s] %s: %s", 
            entry.timestamp.format(TIMESTAMP_FORMAT),
            entry.level,
            entry.message);
    }

    /**
     * Log levels enumeration.
     */
    public enum LogLevel {
        TRACE,
        DEBUG,
        INFO,
        WARN,
        ERROR,
        FATAL
    }

    /**
     * Log entry data class.
     */
    public static class LogEntry {
        public final LogLevel level;
        public final String message;
        public final LocalDateTime timestamp;

        public LogEntry(LogLevel level, String message, LocalDateTime timestamp) {
            this.level = level;
            this.message = message;
            this.timestamp = timestamp;
        }

        @Override
        public String toString() {
            return String.format("LogEntry{level=%s, message='%s', timestamp=%s}", 
                               level, message, timestamp);
        }
    }

    /**
     * Alternative Enum-based Singleton implementation.
     * This is thread-safe by default and provides implicit serialization.
     */
    public enum SingletonLoggerEnum {
        INSTANCE;

        private final ConcurrentLinkedQueue<String> messages = new ConcurrentLinkedQueue<>();

        public void logMessage(String message) {
            String timestampedMessage = LocalDateTime.now().format(TIMESTAMP_FORMAT) + ": " + message;
            messages.offer(timestampedMessage);
            
            // Maintain size limit
            while (messages.size() > MAX_LOG_ENTRIES) {
                messages.poll();
            }
            
            System.out.println("[ENUM-LOGGER] " + timestampedMessage);
        }

        public String[] getMessages() {
            return messages.toArray(new String[0]);
        }

        public void clear() {
            messages.clear();
        }
    }

    /**
     * Prevent cloning of singleton.
     */
    @Override
    protected Object clone() throws CloneNotSupportedException {
        throw new CloneNotSupportedException("Cloning of singleton is not allowed");
    }

    /**
     * Ensure singleton property during deserialization.
     */
    protected Object readResolve() {
        return getInstance();
    }
} 