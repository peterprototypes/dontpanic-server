import { Alert, Grow } from "@mui/material";
import { useFormState } from "react-hook-form";

const FormServerError = (props) => {
  const { errors } = useFormState();

  // if (!errors?.root?.serverError) return false;

  console.log(errors?.root?.serverError?.message);

  return (
    <Grow appear={false} in={Boolean(errors?.root?.serverError?.message)} unmountOnExit {...props}>
      <Alert severity="error">{errors?.root?.serverError?.message}</Alert>
    </Grow>
  );
};

export default FormServerError;