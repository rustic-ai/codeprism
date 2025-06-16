/**
 * Logger - Logging utility with configurable levels
 */

const LOG_LEVELS = {
  ERROR: 0,
  WARN: 1,
  INFO: 2,
  DEBUG: 3,
};

export class Logger {
  constructor(context = 'App') {
    this.context = context;
    this.level = LOG_LEVELS.INFO;
  }
  
  setLevel(level) {
    this.level = typeof level === 'string' ? LOG_LEVELS[level.toUpperCase()] : level;
  }
  
  _log(level, message, data = null) {
    if (level <= this.level) {
      const timestamp = new Date().toISOString();
      const logData = data ? ` ${JSON.stringify(data)}` : '';
      console.log(`[${timestamp}] [${this.context}] ${message}${logData}`);
    }
  }
  
  error(message, data) {
    this._log(LOG_LEVELS.ERROR, `ERROR: ${message}`, data);
  }
  
  warn(message, data) {
    this._log(LOG_LEVELS.WARN, `WARN: ${message}`, data);
  }
  
  info(message, data) {
    this._log(LOG_LEVELS.INFO, `INFO: ${message}`, data);
  }
  
  debug(message, data) {
    this._log(LOG_LEVELS.DEBUG, `DEBUG: ${message}`, data);
  }
} 