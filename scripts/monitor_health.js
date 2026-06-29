#!/usr/bin/env node

/**
 * Health Check Monitoring Script
 * 
 * Monitors Uzima contract health and sends alerts when thresholds are exceeded.
 * 
 * Usage:
 *   node monitor_health.js [--interval=60] [--contracts=contract1,contract2]
 */

const ALERT_THRESHOLDS = {
  successRate: 9500,        // 95% (value is percentage * 100)
  storageUsage: 8192,       // 8KB
  inactivityPeriod: 3600,   // 1 hour in seconds
  errorRate: 50,            // 5% (errors per 1000 operations)
  gasUsageMultiplier: 2.0,  // Alert if 2x average
};

const COLORS = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
};

class HealthMonitor {
  constructor(contracts, interval = 60000) {
    this.contracts = contracts;
    this.interval = interval;
    this.alerts = [];
    this.history = new Map();
  }

  async checkHealth(contractId) {
    try {
      // In production, this would call the actual contract
      // For now, we'll simulate the response
      const health = await this.fetchHealthCheck(contractId);
      
      this.storeHistory(contractId, health);
      const alerts = this.analyzeHealth(contractId, health);
      
      if (alerts.length > 0) {
        this.handleAlerts(contractId, alerts);
      }
      
      return { contractId, health, alerts };
    } catch (error) {
      console.error(`${COLORS.red}Error checking ${contractId}: ${error.message}${COLORS.reset}`);
      return { contractId, error: error.message };
    }
  }

  async fetchHealthCheck(contractId) {
    // Simulate contract call
    // In production: const health = await contract.health_check();
    return {
      version: '1.0.0',
      is_paused: false,
      storage_usage: Math.floor(Math.random() * 10000),
      last_activity: Math.floor(Date.now() / 1000) - Math.floor(Math.random() * 7200),
      total_operations: Math.floor(Math.random() * 100000),
      failed_operations: Math.floor(Math.random() * 1000),
      success_rate: 9500 + Math.floor(Math.random() * 500),
    };
  }

  storeHistory(contractId, health) {
    if (!this.history.has(contractId)) {
      this.history.set(contractId, []);
    }
    
    const history = this.history.get(contractId);
    history.push({
      timestamp: Date.now(),
      health,
    });
    
    // Keep only last 100 entries
    if (history.length > 100) {
      history.shift();
    }
  }

  analyzeHealth(contractId, health) {
    const alerts = [];
    const now = Math.floor(Date.now() / 1000);

    // Check success rate
    if (health.success_rate < ALERT_THRESHOLDS.successRate) {
      alerts.push({
        severity: 'critical',
        metric: 'success_rate',
        message: `Low success rate: ${(health.success_rate / 100).toFixed(2)}%`,
        value: health.success_rate,
        threshold: ALERT_THRESHOLDS.successRate,
      });
    }

    // Check storage usage
    if (health.storage_usage > ALERT_THRESHOLDS.storageUsage) {
      alerts.push({
        severity: 'warning',
        metric: 'storage_usage',
        message: `High storage usage: ${health.storage_usage} bytes`,
        value: health.storage_usage,
        threshold: ALERT_THRESHOLDS.storageUsage,
      });
    }

    // Check inactivity
    const inactivityPeriod = now - health.last_activity;
    if (inactivityPeriod > ALERT_THRESHOLDS.inactivityPeriod) {
      alerts.push({
        severity: 'warning',
        metric: 'inactivity',
        message: `No activity for ${Math.floor(inactivityPeriod / 60)} minutes`,
        value: inactivityPeriod,
        threshold: ALERT_THRESHOLDS.inactivityPeriod,
      });
    }

    // Check pause status
    if (health.is_paused) {
      alerts.push({
        severity: 'critical',
        metric: 'paused',
        message: 'Contract is paused',
        value: true,
        threshold: false,
      });
    }

    // Check error rate
    if (health.total_operations > 0) {
      const errorRate = (health.failed_operations * 1000) / health.total_operations;
      if (errorRate > ALERT_THRESHOLDS.errorRate) {
        alerts.push({
          severity: 'warning',
          metric: 'error_rate',
          message: `High error rate: ${errorRate.toFixed(2)} per 1000 ops`,
          value: errorRate,
          threshold: ALERT_THRESHOLDS.errorRate,
        });
      }
    }

    return alerts;
  }

  handleAlerts(contractId, alerts) {
    for (const alert of alerts) {
      const color = alert.severity === 'critical' ? COLORS.red : COLORS.yellow;
      const timestamp = new Date().toISOString();
      
      console.log(`${color}[${alert.severity.toUpperCase()}] ${timestamp}${COLORS.reset}`);
      console.log(`  Contract: ${contractId}`);
      console.log(`  Metric: ${alert.metric}`);
      console.log(`  ${alert.message}`);
      console.log('');
      
      this.alerts.push({
        timestamp,
        contractId,
        ...alert,
      });
    }
  }

  displayStatus(results) {
    console.clear();
    console.log(`${COLORS.cyan}╔════════════════════════════════════════════════════════════╗${COLORS.reset}`);
    console.log(`${COLORS.cyan}║          Uzima Contract Health Monitor                     ║${COLORS.reset}`);
    console.log(`${COLORS.cyan}╚════════════════════════════════════════════════════════════╝${COLORS.reset}`);
    console.log('');
    console.log(`Last Check: ${new Date().toLocaleString()}`);
    console.log(`Monitoring ${results.length} contracts`);
    console.log('');

    for (const result of results) {
      if (result.error) {
        console.log(`${COLORS.red}✗ ${result.contractId}: ERROR${COLORS.reset}`);
        console.log(`  ${result.error}`);
        continue;
      }

      const { health, alerts } = result;
      const status = alerts.length === 0 ? 
        `${COLORS.green}✓ Healthy${COLORS.reset}` : 
        `${COLORS.red}✗ ${alerts.length} Alert(s)${COLORS.reset}`;

      console.log(`${status} ${COLORS.blue}${result.contractId}${COLORS.reset}`);
      console.log(`  Version: ${health.version}`);
      console.log(`  Success Rate: ${(health.success_rate / 100).toFixed(2)}%`);
      console.log(`  Operations: ${health.total_operations.toLocaleString()} (${health.failed_operations} failed)`);
      console.log(`  Storage: ${health.storage_usage.toLocaleString()} bytes`);
      
      const lastActivity = Math.floor(Date.now() / 1000) - health.last_activity;
      const activityStr = lastActivity < 60 ? 
        `${lastActivity}s ago` : 
        `${Math.floor(lastActivity / 60)}m ago`;
      console.log(`  Last Activity: ${activityStr}`);
      
      if (health.is_paused) {
        console.log(`  ${COLORS.red}⚠ PAUSED${COLORS.reset}`);
      }

      if (alerts.length > 0) {
        console.log(`  ${COLORS.yellow}Alerts:${COLORS.reset}`);
        for (const alert of alerts) {
          console.log(`    - ${alert.message}`);
        }
      }
      
      console.log('');
    }

    // Display recent alerts summary
    if (this.alerts.length > 0) {
      console.log(`${COLORS.yellow}Recent Alerts (last 10):${COLORS.reset}`);
      const recentAlerts = this.alerts.slice(-10);
      for (const alert of recentAlerts) {
        console.log(`  [${alert.timestamp}] ${alert.contractId}: ${alert.message}`);
      }
      console.log('');
    }
  }

  async start() {
    console.log(`${COLORS.green}Starting health monitor...${COLORS.reset}`);
    console.log(`Interval: ${this.interval / 1000}s`);
    console.log(`Contracts: ${this.contracts.join(', ')}`);
    console.log('');

    const monitor = async () => {
      const results = await Promise.all(
        this.contracts.map(contract => this.checkHealth(contract))
      );
      
      this.displayStatus(results);
    };

    // Initial check
    await monitor();

    // Schedule periodic checks
    setInterval(monitor, this.interval);
  }

  generateReport() {
    const report = {
      timestamp: new Date().toISOString(),
      contracts: {},
      summary: {
        total_contracts: this.contracts.length,
        healthy_contracts: 0,
        contracts_with_alerts: 0,
        total_alerts: this.alerts.length,
      },
    };

    for (const contractId of this.contracts) {
      const history = this.history.get(contractId) || [];
      if (history.length === 0) continue;

      const latest = history[history.length - 1].health;
      const hasAlerts = this.analyzeHealth(contractId, latest).length > 0;

      if (!hasAlerts) {
        report.summary.healthy_contracts++;
      } else {
        report.summary.contracts_with_alerts++;
      }

      report.contracts[contractId] = {
        current_health: latest,
        data_points: history.length,
        avg_success_rate: this.calculateAverage(history, 'success_rate'),
        avg_storage_usage: this.calculateAverage(history, 'storage_usage'),
      };
    }

    return report;
  }

  calculateAverage(history, field) {
    if (history.length === 0) return 0;
    const sum = history.reduce((acc, entry) => acc + entry.health[field], 0);
    return sum / history.length;
  }
}

// Parse command line arguments
function parseArgs() {
  const args = process.argv.slice(2);
  const config = {
    interval: 60000, // 1 minute default
    contracts: [
      'appointment_booking_escrow',
      'medical_record_backup',
      'identity_registry',
      'healthcare_payment',
      'audit',
    ],
  };

  for (const arg of args) {
    if (arg.startsWith('--interval=')) {
      config.interval = parseInt(arg.split('=')[1]) * 1000;
    } else if (arg.startsWith('--contracts=')) {
      config.contracts = arg.split('=')[1].split(',');
    }
  }

  return config;
}

// Main execution
if (require.main === module) {
  const config = parseArgs();
  const monitor = new HealthMonitor(config.contracts, config.interval);
  
  monitor.start().catch(error => {
    console.error(`${COLORS.red}Fatal error: ${error.message}${COLORS.reset}`);
    process.exit(1);
  });

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    console.log(`\n${COLORS.yellow}Generating final report...${COLORS.reset}`);
    const report = monitor.generateReport();
    console.log(JSON.stringify(report, null, 2));
    process.exit(0);
  });
}

module.exports = HealthMonitor;
