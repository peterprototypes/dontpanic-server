import { CircularProgress, Box } from "@mui/material";

const LoadingPage = () => {
  return (
    <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%', with: '100%' }}>
      <CircularProgress />
    </Box>
  );
};

export default LoadingPage;