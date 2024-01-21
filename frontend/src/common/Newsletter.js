import '../App.css';
import React, { Component } from 'react';

class Newsletter extends Component {

    render() {
        return(
            <div className='newsletter'>
                <div className='newsletter-inner'>
                    <h2>Stay Connected With Our Email Updates</h2>
                    <p>Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.</p>
                    <form>
                        <input type='email' placeholder='Your email address' name='email' />
                        <button>SIGN UP</button>
                    </form>
                </div>
            </div>
        );
    }

}

export default Newsletter;