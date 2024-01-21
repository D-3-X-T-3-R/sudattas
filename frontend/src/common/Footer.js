import '../App.css';
import React, { Component } from 'react';

import facebook from '../images/facebook.png';
import instagram from '../images/instagram.png';
import linkedin from '../images/linkedin.png';
import twitter from '../images/twitter.png';

class Footer extends Component {

    render() {
        return(
            <div className='footer'>
                <div className='footer-inner'>
                    <div className='footer-copyright'>
                        <p>© 2024 Sudattas</p>
                    </div>
                    <div className='footer-menu'>
                        <ul>
                            <li>
                                <a href="/">Privacy</a>
                            </li>
                            <li>
                                <a href="/">Terms & Conditions</a>
                            </li>
                            <li>
                                <a href="/">Legal</a>
                            </li>
                            <li>
                                <a href="/">Site Map</a>
                            </li>
                        </ul>
                    </div>
                    <div className='footer-social'>
                        <ul>
                            <li>
                                <a href='/'>
                                    <img src={facebook} alt="Facebook" width="100%" />
                                </a>
                            </li>
                            <li>
                                <a href='/'>
                                    <img src={instagram} alt="Instagram" width="100%" />
                                </a>
                            </li>
                            <li>
                                <a href='/'>
                                    <img src={linkedin} alt="Linkedin" width="100%" />
                                </a>
                            </li>
                            <li>
                                <a href='/'>
                                    <img src={twitter} alt="X" width="100%" />
                                </a>
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        );
    }

}

export default Footer;