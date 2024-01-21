import '../App.css';
import React, { Component } from 'react';

// Images
import category1 from '../images/category_1.png';
import category2 from '../images/category_2.png';
import category3 from '../images/category_3.png';
import category4 from '../images/category_4.png';

class FeaturedCategory extends Component {

    render() {
        return (
            <div className='body'>
                <div className="home">
                    {/* <div className='header'>
                        <h1>Shop By Category</h1>
                        <p>Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.</p>
                    </div> */}
                    <div className="category-inner">
                        <div className='category-single'>
                            <div className='category-card'>
                                <img src={category1} alt="Category Name" width="100%" />
                                <div className='category-details'>
                                    <span>
                                        <h3>Category Name</h3>
                                        <p>Lorem Ipsum is simply dummy text of the printing and typesetting industry.</p>
                                    </span>
                                </div>
                            </div>
                        </div>
                        <div className='category-single'>
                            <div className='category-card'>
                                <img src={category2} alt="Category Name" width="100%" />
                                <div className='category-details'>
                                    <span>
                                        <h3>Category Name</h3>
                                        <p>Lorem Ipsum is simply dummy text of the printing and typesetting industry.</p>
                                    </span>
                                </div>
                            </div>
                        </div>
                        <div className='category-single'>
                            <div className='category-card'>
                                <img src={category3} alt="Category Name" width="100%" />
                                <div className='category-details'>
                                    <span>
                                        <h3>Category Name</h3>
                                        <p>Lorem Ipsum is simply dummy text of the printing and typesetting industry.</p>
                                    </span>
                                </div>
                            </div>
                        </div>
                        <div className='category-single'>
                            <div className='category-card'>
                                <img src={category4} alt="Category Name" width="100%" />
                                <div className='category-details'>
                                    <span>
                                        <h3>Category Name</h3>
                                        <p>Lorem Ipsum is simply dummy text of the printing and typesetting industry.</p>
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        );
    }

}

export default FeaturedCategory;