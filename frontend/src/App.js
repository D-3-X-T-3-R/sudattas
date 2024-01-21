import './App.css';
import { BrowserRouter, Routes, Route } from "react-router-dom";

//Common Components
import Navbar from '../src/common/Navbar';
import Newsletter from '../src/common/Newsletter';
import Footer from '../src/common/Footer';
import Copyright from '../src/common/Copyright';

//Pages
import Layout from './layout/Layout';
import Home from './pages/Home';
import Category from './pages/Category';
import ProductDetail from './pages/ProductDetail';
import Cart from './pages/Cart';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Home />} />
          <Route path="category" element={<Category />} />
          <Route path="product" element={<ProductDetail />} />
          <Route path="cart" element={<Cart />} />
          {/* <Route path="*" element={<NoPage />} /> */}
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
