import { styled } from '@mui/material/styles';

const StyledGridOverlay = styled('div')(({ theme }) => ({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'center',
  height: '100%',
  backgroundColor: theme.palette.background.default,
}));

const NoRowsOverlay = ({ primaryText }) => {
  return (
    <StyledGridOverlay>
      {primaryText}
    </StyledGridOverlay>
  );
};

export default NoRowsOverlay;