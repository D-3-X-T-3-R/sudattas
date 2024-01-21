import '../App.css';
import React, { Component } from 'react';
import { Outlet, Link } from "react-router-dom";

//Common Components
import Navbar from '../common/Navbar';
import Newsletter from '../common/Newsletter';
import Footer from '../common/Footer';
import Home from '../pages/Home';

class Layout extends Component {

    render() {
        return(
            <div className="App">
                <Navbar />
                <Outlet />
                <Newsletter />
                <Footer />
            </div>
        );
    }

}

export default Layout;