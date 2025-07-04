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
          A powerful Rust-based code analysis tool that helps developers understand, navigate, 
          and optimize their codebases using advanced parsing and AI-driven insights.
        </p>
        <div className={styles.buttons}>
          <Link
            className="button button--primary button--lg"
            to="/docs/mcp-server/getting-started/installation">
            Get Started
          </Link>
          <Link
            className="button button--secondary button--lg"
            to="/docs/architecture/overview">
            Learn Architecture
          </Link>
          <Link
            className="button button--outline button--lg"
            to="/docs/mcp-server/api-reference">
            API Reference
          </Link>
          <Link
            className="button button--sponsor button--lg"
            href="https://github.com/sponsors/dragonscale-ai"
            style={{backgroundColor: '#EA4AAA', color: 'white', border: 'none'}}>
            ❤️ Sponsor
          </Link>
        </div>
        <div className={styles.quickStart}>
          <p>Quick Start:</p>
          <code>cargo install codeprism</code>
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
              <h3>Multi-Language</h3>
              <p>Support for Rust, Python, JavaScript, TypeScript, and more</p>
            </div>
          </div>
          <div className="col col--3">
            <div className={styles.stat}>
              <h3>AI-Powered</h3>
              <p>Intelligent code analysis and semantic search capabilities</p>
            </div>
          </div>
          <div className="col col--3">
            <div className={styles.stat}>
              <h3>Fast & Efficient</h3>
              <p>Built in Rust for maximum performance and reliability</p>
            </div>
          </div>
          <div className="col col--3">
            <div className={styles.stat}>
              <h3>MCP Compatible</h3>
              <p>Model Context Protocol integration for AI assistants</p>
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
      description="Understand and Analyze Your Codebase with AI. A powerful Rust-based code analysis tool with multi-language support and intelligent insights.">
      <HomepageHeader />
      <main>
        <HomepageStats />
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
