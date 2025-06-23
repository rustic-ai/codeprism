import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Advanced Code Parsing',
    Svg: require('@site/static/img/ai-code-parsing.svg').default,
    description: (
      <>
        Parse and analyze code across multiple languages including Rust, Python, JavaScript, 
        and TypeScript. Extract ASTs, symbols, and dependencies with high precision.
      </>
    ),
  },
  {
    title: 'AI-Powered Analysis',
    Svg: require('@site/static/img/ai-analysis.svg').default,
    description: (
      <>
        Leverage artificial intelligence for semantic code search, complexity analysis, 
        and intelligent insights. Integrate with AI assistants via Model Context Protocol (MCP).
      </>
    ),
  },
  {
    title: 'Performance & Scalability',
    Svg: require('@site/static/img/performance-scalability.svg').default,
    description: (
      <>
        Built in Rust for maximum performance and memory safety. Handle large codebases 
        efficiently with incremental analysis and smart caching strategies.
      </>
    ),
  },
  {
    title: 'Rich API & Tools',
    Svg: require('@site/static/img/api-tools.svg').default,
    description: (
      <>
        Comprehensive REST API, command-line tools, and extensible architecture. 
        Easy integration with editors, CI/CD pipelines, and development workflows.
      </>
    ),
  },
  {
    title: 'Code Quality Insights',
    Svg: require('@site/static/img/code-quality.svg').default,
    description: (
      <>
        Identify code smells, security issues, and performance bottlenecks. 
        Get actionable recommendations to improve code quality and maintainability.
      </>
    ),
  },
  {
    title: 'Developer Experience',
    Svg: require('@site/static/img/developer-experience.svg').default,
    description: (
      <>
        Simple setup, intuitive APIs, and comprehensive documentation. 
        Focus on your code while CodePrism handles the complex analysis tasks.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="text--center margin-bottom--lg">
          <Heading as="h2">Why Choose CodePrism?</Heading>
          <p>Discover the powerful features that make CodePrism the ideal choice for code analysis</p>
        </div>
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
