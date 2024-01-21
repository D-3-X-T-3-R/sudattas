import '../App.css';
import React, { Component } from 'react';

import product1 from '../images/category-image1.jpg';
import product2 from '../images/category-image2.jpg';
import product3 from '../images/category-image3.jpg';
import product4 from '../images/product1.jpg';

function Category() {

    return (
        <div className='body'>
            <div className='page-header'>
                <h2>Category Name</h2>
                <ul>
                    <li>
                        <a href="/">Home</a>
                    </li>
                    <li>
                        <a href="/category">Category</a>
                    </li>
                </ul>
            </div>
            <div className='category'>
                <div className='category-filter'>
                    <div className='filter-single'>
                        <div className='filter-header'>
                            <p>Shop By Category</p>
                        </div>
                        <div className='filter-options'>
                            <ul>
                                <li>
                                    <label>
                                        <input type='checkbox' name='' checked/>
                                        Category 1
                                    </label>
                                    <span className='active'>11</span>
                                </li>
                                <li>
                                    <label>
                                        <input type='checkbox' name='' />
                                        Category 2
                                    </label>
                                    <span>5</span>
                                </li>
                                <li>
                                    <label>
                                        <input type='checkbox' name='' />
                                        Category 3
                                    </label>
                                    <span>11</span>
                                </li>
                                <li>
                                    <label>
                                        <input type='checkbox' name='' />
                                        Category 4
                                    </label>
                                    <span>11</span>
                                </li>
                                <li>
                                    <label>
                                        <input type='checkbox' name='' />
                                        Category 5
                                    </label>
                                    <span>11</span>
                                </li>
                            </ul>
                        </div>
                    </div>
                    <div className='filter-single'>
                        <div className='filter-header'>
                            <p>Shop By Brand</p>
                        </div>
                        <div className='filter-options'>
                            <ul>
                                <li>
                                    <label>
                                        <input type='checkbox' name='' />
                                        Brand 1
                                    </label>
                                    <span>11</span>
                                </li>
                                <li>
                                    <label>
                                        <input type='checkbox' name='' />
                                        Brand 2
                                    </label>
                                    <span>11</span>
                                </li>
                            </ul>
                        </div>
                    </div>
                    <div className='filter-single'>
                        <div className='filter-header'>
                            <p>Shop By Color</p>
                        </div>
                        <div className='filter-options'>
                            <ul>
                                <li>
                                    <label>
                                        <input type='checkbox' name='' />
                                        Color 1
                                    </label>
                                    <span>11</span>
                                </li>
                                <li>
                                    <label>
                                        <input type='checkbox' name='' />
                                        Color 2
                                    </label>
                                    <span>11</span>
                                </li>
                            </ul>
                        </div>
                    </div>
                </div>
                <div className='product-sort'>
                    <label>
                        Sort By
                        <select>
                            <option>Featured</option>
                            <option>Best Selling</option>
                            <option>Price: Low to High</option>
                            <option>Price: High to Low</option>
                        </select>
                    </label>
                </div>
                <div className='product-page-inner'>
                    <div className='product-single'>
                        <img src={product1} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product2} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product3} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product4} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product1} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product2} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product3} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product4} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product1} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product2} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product3} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                    <div className='product-single'>
                        <img src={product4} alt="Product Name" width="100%" />
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <p>Product Name</p>
                        <h4>₹ 5,000</h4>
                        <button className='btn'>Add To Cart</button>
                    </div>
                </div>
            </div>
        </div>
    );

}

export default Category;