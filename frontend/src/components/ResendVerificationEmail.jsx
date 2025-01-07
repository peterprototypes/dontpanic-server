import React from 'react';
import useSWRMutation from 'swr/mutation';
import { LoadingButton } from '@mui/lab';
import { useSnackbar } from 'notistack';

const ResendVerificationEmail = ({ email, initialWait = 60, ...rest }) => {
  const { enqueueSnackbar } = useSnackbar();

  const { trigger, isMutating } = useSWRMutation('/api/auth/resend-verification-email');

  const [waitTime, setWaitTime] = React.useState(initialWait);

  const onSend = () => {
    trigger({ email })
      .then(() => setWaitTime(60))
      .catch((e) => enqueueSnackbar(e.message, { variant: 'error' }));
  };

  React.useEffect(() => {
    if (waitTime > 0) {
      const timer = setTimeout(() => setWaitTime(waitTime - 1), 1000);
      return () => clearTimeout(timer);
    }
  }, [waitTime]);

  return (
    <LoadingButton
      onClick={onSend}
      variant="contained"
      disabled={waitTime > 0}
      loading={isMutating}
      {...rest}
    >
      Resend Verification Email
      {waitTime > 0 && `(${waitTime}s)`}
    </LoadingButton>
  );
};

export default ResendVerificationEmail;