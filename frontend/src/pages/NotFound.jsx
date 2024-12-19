import { Container, GlobalStyles, Typography, Box, Divider, Stack } from "@mui/material";

const NotFound = () => {
  return (
    <Container maxWidth="xs" sx={{ flexGrow: 1, py: 3 }}>
      <Box display="flex" justifyContent="center" alignItems="center" height="100%">
        <Stack spacing={2}>
          <Typography variant="h2">404</Typography>
          <Divider />
          <Typography variant="body1">
            The page you are looking for does not exist. Please check the URL or go back to the homepage.
          </Typography>
        </Stack>
      </Box>
      <GlobalStyles
        styles={{
          body: { backgroundColor: "white" }
        }}
      />
    </Container>
  );
};

export default NotFound;