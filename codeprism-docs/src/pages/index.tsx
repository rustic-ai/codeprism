import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';

import styles from './index.module.css';

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <p className={styles.heroDescription}>
          CodePrism turns sprawling, multi-language repos into a navigable knowledge-graph your AI assistants can reason about. Rust-powered speed, MIT-licensed freedom.
        </p>
        <div className={styles.buttons}>
          <Link
            className="button button--primary button--lg"
            to="/docs/mcp-server/getting-started/installation">
            Get Started
          </Link>
          <Link
            className="button button--secondary button--lg"
            to="/docs/mcp-server/architecture/overview">
            Architecture Deep Dive
          </Link>
          <Link
            className="button button--outline button--lg"
            to="/docs/mcp-server/api-reference">
            API Reference
          </Link>
          <Link
            className="button button--sponsor button--lg"
            href="https://github.com/sponsors/dragonscale-ai"
            style={{backgroundColor: '#ff8a57', color: 'white', border: '1px solid #ff8a57'}}>
            ❤️ Sponsor
          </Link>
        </div>
        <div className={styles.quickStart}>
          <div className="row">
            <div className="col col--6">
              <p><strong>CodePrism Server:</strong></p>
              <code>cargo install codeprism-mcp-server</code>
              <code>codeprism-mcp-server --help</code>
            </div>
            <div className="col col--6">
              <p><strong>Mandrel Test Harness:</strong></p>
              <code>cargo install mandrel-mcp-th</code>
              <code>moth --version</code>
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}

function HomepageStats() {
  return (
    <section className={styles.stats}>
      <div className="container">
        <div className="row">
          <div className="col col--3">
            <div className={styles.stat}>
              <h3>5 Languages & Counting</h3>
              <p>Rust, Python, JavaScript, TypeScript, Java – plus pluggable adapters.</p>
            </div>
          </div>
          <div className="col col--3">
            <div className={styles.stat}>
              <h3>100% AI-Generated Core</h3>
              <p>Built entirely with AI-first engineering practices.</p>
            </div>
          </div>
          <div className="col col--3">
            <div className={styles.stat}>
              <h3>&lt; 50ms Query Latency</h3>
              <p>Lightweight graph store delivers sub-second answers.</p>
            </div>
          </div>
          <div className="col col--3">
            <div className={styles.stat}>
              <h3>MCP Native</h3>
              <p>First-class Model Context Protocol implementation for agent integration.</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function CodePrismSection() {
  return (
    <section className={`${styles.codeprismSection} codeprismSection`}>
      <div className="container">
        <div className="row">
          <div className="col col--6">
            <Heading as="h2">CodePrism Server</Heading>
            <h3>Graph-First Code Intelligence for AI Assistants</h3>
            <p>
              CodePrism transforms your codebase into an interactive knowledge graph that AI assistants can navigate and understand. 
              Built with Rust for performance, it implements the Model Context Protocol (MCP) to provide standardized code intelligence 
              across multiple programming languages. Available on crates.io and GitHub releases for instant installation.
            </p>
            <ul>
              <li><strong>20+ Production Tools</strong> - From symbol search to complexity analysis</li>
              <li><strong>Multi-Language Support</strong> - JavaScript, TypeScript, Python, Rust, Java</li>
              <li><strong>Real-time Updates</strong> - Graph updates as your code changes</li>
              <li><strong>Enterprise Ready</strong> - Production-tested with 100% tool success rate</li>
            </ul>
            <div className={`${styles.buttonGroup} buttonGroup`}>
              <Link className="button button--primary" to="/docs/mcp-server/getting-started/installation">
                Get Started
              </Link>
              <Link className="button button--outline" to="/docs/mcp-server/overview">
                Learn More
              </Link>
            </div>
          </div>
          <div className="col col--6">
            <div className={styles.codeExample}>
              <h4>Ready in Minutes</h4>
              <pre><code>{`# Install from crates.io
$ cargo install codeprism-mcp-server

# Start the MCP server
$ codeprism-mcp-server

# Example MCP tool schema
{
  "name": "analyze_complexity",
  "description": "Analyze code complexity",
  "inputSchema": {
    "type": "object", 
    "properties": {
      "target": {
        "type": "string",
        "description": "File or function to analyze"
      }
    }
  }
}`}</code></pre>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function MandrelSection() {
  return (
    <section className={`${styles.mandrelSection} mandrelSection`}>
      <div className="container">
        <div className="row">
          <div className="col col--6">
            <div className={styles.terminalExample}>
              <h4>Get Started in 30 Seconds</h4>
              <pre><code>{`# Install from crates.io
$ cargo install mandrel-mcp-th

# Create test specification
$ cat > filesystem-test.yaml << EOF
name: "Filesystem MCP Server"
server:
  command: "node"
  args: ["server.js"]
  transport: "stdio"
tools:
  - name: "read_file"
    description: "Read file contents"
EOF

# Run comprehensive tests
$ moth run filesystem-test.yaml

✅ Protocol Compliance: PASSED
✅ Tool Validation: PASSED  
✅ Error Handling: PASSED
📊 Coverage: 95% (19/20 tests)`}</code></pre>
            </div>
          </div>
          <div className="col col--6">
            <Heading as="h2">Mandrel MCP Test Harness</Heading>
            <h3>Universal Testing Framework for MCP Servers</h3>
            <p>
              Mandrel (aka <code>moth</code>) is a comprehensive testing framework that validates any MCP server 
              implementation for protocol compliance, functional correctness, and performance characteristics. 
              Built on the official Rust MCP SDK for guaranteed accuracy. Install instantly from crates.io or GitHub releases.
            </p>
            <ul>
              <li><strong>Protocol Compliance</strong> - Full MCP 2025-06-18 specification validation</li>
              <li><strong>Performance Testing</strong> - Concurrent execution with configurable limits</li>
              <li><strong>Rich Reporting</strong> - HTML, JSON, and JUnit XML output formats</li>
              <li><strong>CI/CD Ready</strong> - GitHub Actions, GitLab CI, Jenkins integration</li>
            </ul>
            <div className={`${styles.buttonGroup} buttonGroup`}>
              <Link className="button button--primary" to="/docs/test-harness/getting-started/quick-start">
                Quick Start
              </Link>
              <Link className="button button--outline" to="/docs/test-harness/">
                Documentation
              </Link>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title} - Code Analysis Tool`}
      description="Open-source AI Code Intelligence for Every Codebase. Graph-powered analysis, instant insights, limitless extensibility.">
      <HomepageHeader />
      <main>
        <HomepageStats />
        <CodePrismSection />
        <MandrelSection />
        <HomepageFeatures />
      </main>
    </Layout>
  );
}