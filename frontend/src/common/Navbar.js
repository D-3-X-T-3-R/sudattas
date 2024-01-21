import '../App.css';
import React, { Component } from 'react';
import { Link, useNavigate } from "react-router-dom";

// Images
import logo from '../images/logo.jpg';
import search from '../images/search.png';
import user from '../images/user.png';
import cart from '../images/cart.png';

function Navbar() {

    const navigate = useNavigate();

    const goToCart = () => {
        // alert("Go To Cart");
        navigate("/cart");
    }

    return(
        <div className='navbar'>
            <div className='navbar-inner'>
                <div className='navbar-menu'>
                    <ul>
                        <li>
                            <Link to="/">Home</Link>
                        </li>
                        <li>
                            <Link to="/about">About</Link>
                        </li>
                        <li>
                            <a href='/'>Shop <span className='fa fa-caret-down'></span></a>
                            <ul>
                                <li>
                                    <Link to="/category">Category 1</Link>
                                </li>
                                <li>
                                    <Link to="/category">Category 2</Link>
                                </li>
                                <li>
                                    <Link to="/category">Category 3</Link>
                                </li>
                                <li>
                                    <Link to="/category">Category 4</Link>
                                </li>
                            </ul>
                        </li>
                        <li>
                            <Link to="/">Contact</Link>
                        </li>
                    </ul>
                </div>
                <div className='navbar-logo'>
                    <img src={ logo } alt="Logo" width="100%"/>
                </div>
                <div className='navbar-right'>
                    <ul>
                        <li>
                            <button>
                                <img src={ search } alt="Search" width="100%"/>
                            </button>
                        </li>
                        <li>
                            <button>
                                <img src={ user } alt="User" width="100%"/>
                            </button>
                        </li>
                        <li>
                            <button onClick={goToCart}>
                                <img src={ cart } alt="User" width="100%"/>
                            </button>
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    );

}

export default Navbar;