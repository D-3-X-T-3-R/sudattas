import '../App.css';
import React, { Component } from 'react';

// Images
import product1 from '../images/category-image1.jpg';
import product2 from '../images/category-image2.jpg';
import product3 from '../images/category-image3.jpg';
import product4 from '../images/product1.jpg';

class ProductDetail extends Component {

    render() {
        return (
            <div className='body'>
                <div className='product-data-top'>
                    <div className='product-detail-image'>
                        <img src={product1} alt="Product Name" width="100%" />
                        <div className='product-detail-small'>
                            <img src={product1} alt="Product Name" width="100%" />
                            <img src={product2} alt="Product Name" width="100%" />
                            <img src={product3} alt="Product Name" width="100%" />
                            <img src={product4} alt="Product Name" width="100%" />
                        </div>
                    </div>
                    <div className='product-detail-write'>
                        <h1>Product Name</h1>
                        <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi sollicitudin lobortis tempus. Fusce enim quam, dignissim tempus auctor non, blandit at neque. Pellentesque posuere sed purus at aliquam. Cras vitae eros nisi.</p>
                        <ul>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star'></li>
                            <li className='fa fa-star-o'></li>
                        </ul>
                        <h4>₹ 5,000</h4>
                        <div className='product-detail-btn'>
                            <button>Add To Cart</button>
                            <button>Add To Wishlist</button>
                        </div>
                        <div className='product-detail-variant'>
                            <span>Select Color</span>
                        </div>
                        <div className='product-detail-variant'>
                            <span>Select Size</span>
                        </div>
                        <div className='product-detail-social'>
                            <span>Social Share</span>
                            <ul>
                                <li>
                                    <a href='/' className='fa fa-facebook'></a>
                                </li>
                                <li>
                                    <a href='/' className='fa fa-instagram'></a>
                                </li>
                                <li>
                                    <a href='/' className='fa fa-whatsapp'></a>
                                </li>
                                <li>
                                    <a href='/' className='fa fa-pinterest'></a>
                                </li>
                            </ul>
                        </div>
                    </div>
                </div>
                <div className='product-data-bottom'>
                    <h2>Product Description</h2>
                    <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi sollicitudin lobortis tempus. Fusce enim quam, dignissim tempus auctor non, blandit at neque. Pellentesque posuere sed purus at aliquam. Cras vitae eros nisi. Nunc eu sodales neque. Maecenas ac varius nulla. Fusce ac sollicitudin felis, sed molestie velit. Morbi tincidunt quam sed urna pellentesque, eget sagittis sapien congue. Maecenas commodo turpis nec lectus posuere, sed placerat lacus suscipit. Aliquam efficitur tincidunt elit eget sagittis. Fusce quis lobortis orci.</p>

                    <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi sollicitudin lobortis tempus. Fusce enim quam, dignissim tempus auctor non, blandit at neque. Pellentesque posuere sed purus at aliquam. Cras vitae eros nisi. Nunc eu sodales neque. Maecenas ac varius nulla. Fusce ac sollicitudin felis, sed molestie velit. Morbi tincidunt quam sed urna pellentesque, eget sagittis sapien congue. Maecenas commodo turpis nec lectus posuere, sed placerat lacus suscipit. Aliquam efficitur tincidunt elit eget sagittis. Fusce quis lobortis orci.</p>

                    <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi sollicitudin lobortis tempus. Fusce enim quam, dignissim tempus auctor non, blandit at neque. Pellentesque posuere sed purus at aliquam. Cras vitae eros nisi. Nunc eu sodales neque. Maecenas ac varius nulla. Fusce ac sollicitudin felis, sed molestie velit. Morbi tincidunt quam sed urna pellentesque, eget sagittis sapien congue. Maecenas commodo turpis nec lectus posuere, sed placerat lacus suscipit. Aliquam efficitur tincidunt elit eget sagittis. Fusce quis lobortis orci.</p>
                </div>
                <div className="home">
                    <div className='header'>
                        <h1>Related Products</h1>
                        <p>Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.</p>
                    </div>
                    <div className='product-inner'>
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

}

export default ProductDetail;