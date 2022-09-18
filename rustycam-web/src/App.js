import './App.css';
import CamInfo from './components/CamInfo';
import Home from './components/Home';
import {
  BrowserRouter,
  Routes,
  Route,
} from "react-router-dom";



function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home/>}>
        <Route path="/info" element={<CamInfo/>}/>
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
