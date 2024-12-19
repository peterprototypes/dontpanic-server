import { Container, GlobalStyles, Typography, Box, Divider, Stack, Link } from "@mui/material";

const ErrorFallback = ({ error }) => {
  return (
    <Container maxWidth="md" sx={{ flexGrow: 1, py: 3 }}>
      <Box display="flex" justifyContent="center" alignItems="center" height="100%">
        <Stack spacing={2}>
          <Typography variant="h2">Oops</Typography>
          <Divider />
          <Typography variant="body1">
            We encountered an error. Please try again later or <Link href="https://github.com/peterprototypes/dontpanic-server/issues">open an issue</Link>.
          </Typography>
          {error?.message && (
            <>
              <Typography color="error">
                {error.message}
              </Typography>
              {error?.stack && (
                <Typography component="pre" variant="body2" color="textSecondary" sx={{ whiteSpace: 'pre-wrap', lineHeight: 2 }}>
                  {error.stack}
                </Typography>
              )}
            </>
          )}
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

export default ErrorFallback;