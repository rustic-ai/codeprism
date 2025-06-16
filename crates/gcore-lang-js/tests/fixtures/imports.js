// Test file with various import styles

import React from 'react';
import { useState, useEffect } from 'react';
import * as utils from './utils';
import './styles.css';

export function App() {
    const [count, setCount] = useState(0);
    
    useEffect(() => {
        console.log('Component mounted');
    }, []);
    
    return React.createElement('div', null, `Count: ${count}`);
}

export default App; 