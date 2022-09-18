import React from 'react';
import * as ReactDOMClient from 'react-dom/client';

import store from './store'
import { Provider } from 'react-redux'

import App from './App';

if (process.env.NODE_ENV === 'development') {
  const { worker } = require('./mocks/browser')
  worker.start()
}


ReactDOMClient.createRoot(document.getElementById("root")).render(
  <Provider store={store}>  
    <App />
  </Provider>
);