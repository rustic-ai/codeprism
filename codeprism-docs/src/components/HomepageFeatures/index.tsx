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
    title: 'Advanced Parse Graph',
    Svg: require('@site/static/img/ai-code-parsing.svg').default,
    description: (
      <>
        Precise ASTs & cross-language symbol links stored in a lightning-fast graph.
      </>
    ),
  },
  {
    title: 'AI-Optimised Metrics',
    Svg: require('@site/static/img/ai-analysis.svg').default,
    description: (
      <>
        Complexity, duplication & "hot-spot" scoring designed for LLM consumption.
      </>
    ),
  },
  {
    title: 'Rust-Powered Speed',
    Svg: require('@site/static/img/performance-scalability.svg').default,
    description: (
      <>
        Native performance, tiny memory footprint, zero GC pauses.
      </>
    ),
  },
  {
    title: 'Open API Surface',
    Svg: require('@site/static/img/api-tools.svg').default,
    description: (
      <>
        REST, WebSocket, MCP JSON-RPC – integrate from any stack.
      </>
    ),
  },
  {
    title: 'First-class Plug-ins',
    Svg: require('@site/static/img/code-quality.svg').default,
    description: (
      <>
        Add new languages or analytics in a single WASM bundle.
      </>
    ),
  },
  {
    title: 'Built for OSS',
    Svg: require('@site/static/img/developer-experience.svg').default,
    description: (
      <>
        MIT license, transparent roadmap, community-driven extensions.
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
