import Box from '@mui/material/Box';

import logo from 'assets/dontpanic-ferris-logo.svg';

const Logo = (props) => {
  return (
    <Box component="img" src={logo} alt="Don't Panic Ferris Logo" {...props} />
  );
};

export default Logo;