import { Container, Box, Stack, Typography, Link, Divider } from "@mui/material";

import { useConfig } from "context/config";

const Footer = () => {
  const { config } = useConfig();

  return (
    <Box sx={{ backgroundColor: 'accentBackground', py: '30px', fontSize: 13 }}>
      <Container maxWidth="md">
        <Stack direction="row" justifyContent="space-between">
          <Stack direction="row" spacing={3}>
            <Link href="https://dontpanic.rs/terms" target="_blank">Terms of Use</Link>
            <Divider orientation="vertical" flexItem />
            <Link href="https://dontpanic.rs/privacy" target="_blank">Privacy Policy</Link>
            <Divider orientation="vertical" flexItem />
            <Link href="https://dontpanic.rs/contact" target="_blank">Contact Us</Link>
          </Stack>
          <Typography color="textSecondary" fontSize={13}>
            Copyright &copy; {new Date().getFullYear()} Don&lsquo;t Panic. {config?.version}
          </Typography>
        </Stack>
      </Container>
    </Box >
  );
};

export default Footer;