import '../App.css';
import React, { Component } from 'react';

//Components
import FeaturedCategory from '../components/FeaturedCategory';
import FeatureProduct from '../components/FeaturedProduct';

//Images
import banner from '../images/cms-banner.jpg'

class Home extends Component {

    render() {
        return(
            <div className='body'>
                <div className='banner'>
                    <img src={banner} alt="Product Name" width="100%" />
                </div>
                <FeaturedCategory />
                <FeatureProduct />
            </div>
        );
    }

}

export default Home;